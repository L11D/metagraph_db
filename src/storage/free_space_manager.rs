use std::sync::Arc;

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::storage::{
    StorageError,
    buffer_pool_manager::{BufferPoolManager, FullPageId},
    common::{PAGE_SIZE, PageDataType},
    page_view::FixedPageView,
};

/// Сколько байт свободного места кодирует одна единица в FSM
const FSM_GRANULARITY: usize = PAGE_SIZE / 255;

#[repr(C)]
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Debug, Clone, Copy, PartialEq)]
pub struct FixedFsmEntry {
    /// Битовая маска: бит=1 означает "страница ЗАНЯТА", бит=0 - "страница СВОБОДНА".
    /// Таким образом, зануленная страница состоит только из свободных слотов.
    pub allocated_bitmap: u64,
}
impl FixedFsmEntry {
    pub const PAGES_PER_ENTRY: usize = 64;

    pub fn has_free(&self) -> bool {
        // Если маска не заполнена единицами полностью, есть свободные места
        self.allocated_bitmap != u64::MAX
    }

    pub fn first_free(&self) -> Option<u32> {
        if self.allocated_bitmap == u64::MAX {
            None
        } else {
            // Ищем первый нулевой бит (инвертируем и ищем первый единичный)
            Some((!self.allocated_bitmap).trailing_zeros())
        }
    }

    pub fn set_free(&mut self, local_idx: u32) {
        self.allocated_bitmap &= !(1u64 << local_idx);
    }

    pub fn set_full(&mut self, local_idx: u32) {
        self.allocated_bitmap |= 1u64 << local_idx;
    }
}

#[repr(C)]
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Debug, Clone, Copy, PartialEq)]
pub struct SlottedFsmEntry {
    /// Квантизованный объём ЗАНЯТОГО места: used_bytes / FSM_GRANULARITY.
    /// 0 означает, что на странице 0 занятых байт (т.е. она полностью свободна).
    pub used_space_quantized: u8,
}
impl SlottedFsmEntry {
    pub fn from_free_bytes(free_bytes: usize) -> Self {
        let used_bytes = PAGE_SIZE.saturating_sub(free_bytes);
        let quantized = (used_bytes / FSM_GRANULARITY).min(255) as u8;
        Self {
            used_space_quantized: quantized,
        }
    }

    pub fn min_free_bytes(&self) -> usize {
        let used_bytes = self.used_space_quantized as usize * FSM_GRANULARITY;
        PAGE_SIZE.saturating_sub(used_bytes)
    }
}

/// Сколько Fixed-страниц отслеживает одна FSM-страница
pub const FIXED_FSM_CAPACITY: usize =
    FixedPageView::<&[u8], FixedFsmEntry>::capacity() * FixedFsmEntry::PAGES_PER_ENTRY;

/// Сколько Slotted-страниц отслеживает одна FSM-страница  
pub const SLOTTED_FSM_CAPACITY: usize = FixedPageView::<&[u8], SlottedFsmEntry>::capacity();

// ---------------------------------------------------------------------------
// Free Space Manager
// ---------------------------------------------------------------------------

pub struct FreeSpaceManager {
}

impl FreeSpaceManager {

    pub fn find_free_fixed_page(
        bpm: Arc<BufferPoolManager>,
        fsm_page_type: PageDataType,
    ) -> Result<u32, StorageError> {
        let mut fsm_idx = 0;

        loop {
            let fsm_id = FullPageId {
                page_type: fsm_page_type,
                page_id: fsm_idx,
            };

            // Если fsm_idx выходит за пределы файла, BPM сам выделит страницу и заполнит её нулями
            let guard = bpm.get_page(fsm_id)?;
            let data = guard.data();
            let view = FixedPageView::<&[u8], FixedFsmEntry>::new_read(data.as_ref());

            for entry_idx in 0..FixedPageView::<&[u8], FixedFsmEntry>::capacity() as u16 {
                if let Some(entry) = view.get(entry_idx)? {
                    if let Some(local_page) = entry.first_free() {
                        return Ok(fsm_idx * FIXED_FSM_CAPACITY as u32
                            + entry_idx as u32 * FixedFsmEntry::PAGES_PER_ENTRY as u32
                            + local_page);
                    }
                } else {
                    // Записи нет (header bit == 0). Так как 0 = свободно,
                    // весь блок из 64 страниц полностью свободен по умолчанию.
                    return Ok(fsm_idx * FIXED_FSM_CAPACITY as u32
                        + entry_idx as u32 * FixedFsmEntry::PAGES_PER_ENTRY as u32);
                }
            }

            fsm_idx += 1;
        }
    }

    pub fn find_free_slotted_page(
        bpm: Arc<BufferPoolManager>,
        fsm_page_type: PageDataType,
        needed_bytes: usize,
    ) -> Result<u32, StorageError> {
        let mut fsm_idx = 0;

        // Переводим needed_bytes в максимально допустимое занятое пространство
        let needed_used_max = PAGE_SIZE.saturating_sub(needed_bytes);
        let needed_quantized_max = (needed_used_max / FSM_GRANULARITY).min(255) as u8;

        loop {
            let fsm_id = FullPageId {
                page_type: fsm_page_type,
                page_id: fsm_idx,
            };

            let guard = bpm.get_page(fsm_id)?;
            let data = guard.data();
            let view = FixedPageView::<&[u8], SlottedFsmEntry>::new_read(data.as_ref());

            for entry_idx in 0..FixedPageView::<&[u8], SlottedFsmEntry>::capacity() as u16 {
                if let Some(entry) = view.get(entry_idx)? {
                    // Ищем страницу, где занятого места МЕНЬШЕ или РАВНО допустимому максимуму
                    if entry.used_space_quantized <= needed_quantized_max {
                        return Ok(fsm_idx * SLOTTED_FSM_CAPACITY as u32 + entry_idx as u32);
                    }
                } else {
                    // Если записи нет, страница гарантированно пустая (0 байт занято)
                    return Ok(fsm_idx * SLOTTED_FSM_CAPACITY as u32 + entry_idx as u32);
                }
            }

            fsm_idx += 1;
        }
    }

    pub fn update_fixed(
        bpm: Arc<BufferPoolManager>,
        fsm_page_type: PageDataType,
        page_id: u32,
        has_free: bool,
    ) -> Result<(), StorageError> {
        let (fsm_page_idx, entry_idx, local_idx) = Self::fixed_coords(page_id);
        let fsm_id = FullPageId {
            page_type: fsm_page_type,
            page_id: fsm_page_idx,
        };

        let guard = bpm.get_page(fsm_id)?;
        let mut data = guard.data_mut();
        let mut view = FixedPageView::<&mut [u8], FixedFsmEntry>::new_write(data.as_mut());

        if view.get(entry_idx)?.is_none() {
            // Если записи не было, по умолчанию маска равна 0 (всё свободно)
            let mut new_entry = FixedFsmEntry {
                allocated_bitmap: 0,
            };
            if !has_free {
                new_entry.set_full(local_idx);
            }
            // Используем set() чтобы привязать запись строго к нужному индексу
            view.set(entry_idx, new_entry);
        } else {
            let entry = view.get_mut(entry_idx)?.unwrap();
            if has_free {
                entry.set_free(local_idx);
            } else {
                entry.set_full(local_idx);
            }
        }

        Ok(())
    }

    pub fn update_slotted(
        bpm: Arc<BufferPoolManager>,
        fsm_page_type: PageDataType,
        page_id: u32,
        free_bytes: usize,
    ) -> Result<(), StorageError> {
        let (fsm_page_idx, slot_idx) = Self::slotted_coords(page_id);
        let fsm_id = FullPageId {
            page_type: fsm_page_type,
            page_id: fsm_page_idx,
        };

        let guard = bpm.get_page(fsm_id)?;
        let mut data = guard.data_mut();
        let mut view = FixedPageView::<&mut [u8], SlottedFsmEntry>::new_write(data.as_mut());

        let new_entry = SlottedFsmEntry::from_free_bytes(free_bytes);

        if view.get(slot_idx)?.is_none() {
            view.set(slot_idx, new_entry)?;
        } else {
            *view.get_mut(slot_idx)?.unwrap() = new_entry;
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Внутренние вспомогательные методы
    // -----------------------------------------------------------------------

    /// Разложить global page_id для Fixed на координаты в FSM:
    /// (fsm_page_idx, entry_idx_in_fsm_page, local_page_idx_in_entry)
    fn fixed_coords(page_id: u32) -> (u32, u16, u32) {
        let fsm_page_idx = page_id / FIXED_FSM_CAPACITY as u32;
        let rem = page_id % FIXED_FSM_CAPACITY as u32;
        let entry_idx = (rem / FixedFsmEntry::PAGES_PER_ENTRY as u32) as u16;
        let local_idx = rem % FixedFsmEntry::PAGES_PER_ENTRY as u32;
        (fsm_page_idx, entry_idx, local_idx)
    }

    /// Разложить global page_id для Slotted на координаты в FSM:
    /// (fsm_page_idx, slot_idx_in_fsm_page)
    fn slotted_coords(page_id: u32) -> (u32, u16) {
        let fsm_page_idx = page_id / SLOTTED_FSM_CAPACITY as u32;
        let slot_idx = (page_id % SLOTTED_FSM_CAPACITY as u32) as u16;
        (fsm_page_idx, slot_idx)
    }
}
