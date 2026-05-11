use std::{backtrace::Backtrace, marker::PhantomData};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Ref};
use bincode::{Encode, Decode};

use crate::{bincode::Bincode, storage::{StorageError, common::PAGE_SIZE}};

#[repr(C)]
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Debug, Clone, Copy)]
struct PageHeader {
    slot_count: u16,
    free_space_pointer: u16,
}

#[repr(C)]
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Debug, Clone, Copy)]
struct Slot {
    offset: u16,
    length: u16,
}

const PAGE_HEADER_SIZE: usize = std::mem::size_of::<PageHeader>();
const SLOT_SIZE: usize = std::mem::size_of::<Slot>();
pub const MAX_RECORD_SIZE: usize = PAGE_SIZE - PAGE_HEADER_SIZE - SLOT_SIZE;

pub struct SlottedPageView<B, T> {
    buffer: B,
    _marker: PhantomData<T>,
}

impl<'a, T> SlottedPageView<&'a [u8], T> {
    pub fn new_read(buf: &'a [u8]) -> Self {
        Self { buffer: buf, _marker: PhantomData }
    }
}

impl<'a, T> SlottedPageView<&'a mut [u8], T> {
    pub fn new_write(buf: &'a mut [u8]) -> Self {
        Self { buffer: buf, _marker: PhantomData }
    }
}

pub fn init_header(buf: &mut [u8]) {
    let mut header: Ref<&mut [u8], PageHeader> = Ref::from_bytes( &mut buf.as_mut()[..PAGE_HEADER_SIZE]).unwrap();
    header.slot_count = 0;
    header.free_space_pointer = PAGE_SIZE as u16;
}

// --- Общие методы (Read-only) ---
impl<B, T> SlottedPageView<B, T>
where
    B: AsRef<[u8]>,
    T: Decode<()>,
{
    pub fn new(buffer: B) -> Self {
        Self { buffer, _marker: PhantomData }
    }

    fn header(&self) -> Ref<&[u8], PageHeader> {
        Ref::from_bytes(&self.buffer.as_ref()[..PAGE_HEADER_SIZE]).unwrap()
    }

    fn slot_offset(slot_id: u16) -> usize {
        PAGE_HEADER_SIZE + slot_id as usize * SLOT_SIZE
    }

    fn slot(&self, slot_id: u16) -> Result<Ref<&[u8], Slot>, StorageError> {
        let header = self.header();
        if slot_id >= header.slot_count { 
            return Err(StorageError::SlotOutOfRange(slot_id, Backtrace::capture()));
        }
        
        let offset = Self::slot_offset(slot_id);
        Ok(Ref::from_bytes(&self.buffer.as_ref()[offset..offset + SLOT_SIZE]).unwrap())
    }

    /// Получить и десериализовать объект из слота
    pub fn get(&self, slot_id: u16) -> Result<Option<T>, StorageError> {
        let slot = self.slot(slot_id)?;
        if slot.length == 0 { return Ok(None); } // Tombstone

        let start = slot.offset as usize;
        let end = start + slot.length as usize;
        let data = &self.buffer.as_ref()[start..end];

        // Десериализация bincode
        let decoded: T = Bincode::decode_from_slice(data);
        Ok(Some(decoded))
    }

    pub fn free_space(&self) -> usize {
        let h = self.header();
        let slots_area_end = PAGE_HEADER_SIZE + (h.slot_count as usize * SLOT_SIZE);
        (h.free_space_pointer as usize).saturating_sub(slots_area_end)
    }
}

// --- Методы только для записи (Mutable) ---
impl<B, T> SlottedPageView<B, T>
where
    B: AsMut<[u8]> + AsRef<[u8]>,
    T: Encode + Decode<()>,
{
    fn header_mut(&mut self) -> Ref<&mut [u8], PageHeader> {
        Ref::from_bytes(&mut self.buffer.as_mut()[..PAGE_HEADER_SIZE]).unwrap()
    }

    fn slot_mut(&mut self, slot_id: u16) -> Result<Ref<&mut [u8], Slot>, StorageError> {
        let count = self.header().slot_count;
        if slot_id >= count { 
            return Err(StorageError::SlotOutOfRange(slot_id, Backtrace::capture()));
        }
        
        let offset = Self::slot_offset(slot_id);
        Ok(Ref::from_bytes(&mut self.buffer.as_mut()[offset..offset + SLOT_SIZE]).unwrap())
    }

    pub fn insert(&mut self, data: &[u8]) -> Result<u16, StorageError> {
        // let data = Bincode::encode_to_vec(item);
        let encoded_len = data.len();

        // 2. Проверяем место (нужен новый слот + сами данные)
        if self.free_space() < (SLOT_SIZE + encoded_len) {
            // Недостаточно места, пробуем копакт
            self.compact();
            if self.free_space() < (SLOT_SIZE + encoded_len) {
                return Err(StorageError::PageFull);
            }
        }

        // 3. Сериализуем данные в конец свободной области
        let new_end = self.header().free_space_pointer as usize;
        let new_start = new_end - encoded_len;

        self.buffer.as_mut()[new_start..new_end].copy_from_slice(&data);

        // 4. Обновляем слот
        let slot_id = self.header().slot_count;

        {
            let mut header = self.header_mut();
            header.slot_count += 1;
            header.free_space_pointer = new_start as u16;
        }

        {
            let mut slot = self.slot_mut(slot_id).unwrap();
            slot.offset = new_start as u16;
            slot.length = encoded_len as u16;
        }


        Ok(slot_id)
    }

    /// В option возвращаем новый slot_id, если запись была перемещена, иначе None
    pub fn update(&mut self, slot_id: u16, item: &T) -> Result<u16, StorageError> {
        let data = Bincode::encode_to_vec(item);
        let encoded_len = data.len();

        let slot = self.slot(slot_id)?;

        // Если запись удалена
        // if slot.length == 0 {
        //     return Ok(None)
        // }

        if data.len() as u16 <= slot.length {
            // Перезаписываем на месте
            let offset = slot.offset as usize;
            self.buffer.as_mut()[offset..offset + encoded_len].copy_from_slice(&data);
            self.slot_mut(slot_id).unwrap().length = encoded_len as u16;
            return Ok(slot_id)
        } else {
            let data = Bincode::encode_to_vec(item);
            match self.insert(&data) {
                Ok(new_slot) => {
                    self.delete(slot_id)?;
                    return Ok(new_slot)
                },
                Err(e) => return Err(e),
            }
        }
    }

    pub fn delete(&mut self, slot_id: u16) -> Result<(), StorageError> {
        let mut slot =  self.slot_mut(slot_id)?;
        slot.length = 0; // Tombstone
        Ok(())
    }

    pub fn compact(&mut self) {
        let slot_count = self.header().slot_count;
        let mut free_ptr = PAGE_SIZE as u16;

        // Идем с конца страницы
        for slot_id in 0..slot_count {
            let slot = self.slot(slot_id).unwrap();

            if slot.length == 0 {
                continue;
            }

            let old_offset = slot.offset;
            let len = slot.length;

            free_ptr -= len;

            // Перемещаем данные
            let src_start = old_offset as usize;
            let src_end = (old_offset + len) as usize;
            

            self.buffer.as_mut()
                .copy_within(src_start..src_end, free_ptr as usize);

            // Обновляем offset
            self.slot_mut(slot_id).unwrap().offset = free_ptr;
        }

        // Обновляем header
        let mut header = self.header_mut();
        header.free_space_pointer = free_ptr;
    }

    // pub fn compact2(&mut self) {
    //     let slot_count = self.header().slot_count;
    //     let mut current_free_ptr = PAGE_SIZE as u16;

    //     for id in 0..slot_count {
    //         let (old_offset, len) = {
    //             let s = self.slot(id).unwrap();
    //             (s.offset, s.length)
    //         };

    //         if len == 0 { continue; }

    //         current_free_ptr -= len;
    //         if old_offset != current_free_ptr {
    //             self.buffer.as_mut().copy_within(
    //                 old_offset as usize .. (old_offset + len) as usize, 
    //                 current_free_ptr as usize
    //             );
    //             self.slot_mut(id).unwrap().offset = current_free_ptr;
    //         }
    //     }
    //     self.header_mut().free_space_pointer = current_free_ptr;
    // }

}