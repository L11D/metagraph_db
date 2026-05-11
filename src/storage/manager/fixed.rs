use std::sync::Arc;

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::storage::{
    SlottedRecordId, StorageError, buffer_pool_manager::{BufferPoolManager, FullPageId}, common::PageDataType, free_space_manager::FreeSpaceManager, page_view::FixedPageView
};

pub trait FixedRecord:
    FromBytes + IntoBytes + KnownLayout + Immutable + Clone + Copy + PartialEq
{
    const PAGE_DATA_TYPE: PageDataType;
    const FSM_PAGE_DATA_TYPE: PageDataType;
}

pub struct FixedRecordManager {
    bpm: Arc<BufferPoolManager>,
}

impl FixedRecordManager {
    pub fn new(bpm: Arc<BufferPoolManager>) -> Self {
        Self { bpm }
    }

    pub fn insert<R: FixedRecord>(&self, record: R) -> Result<SlottedRecordId, StorageError> {
        loop {
            let free_page_idx = FreeSpaceManager::find_free_fixed_page(
                self.bpm.clone(),
                R::FSM_PAGE_DATA_TYPE,
            )?;

            let full_id = FullPageId {
                page_type: R::PAGE_DATA_TYPE,
                page_id: free_page_idx,
            };

            let page = self.bpm.get_page(full_id)?;
            let mut page_data = page.data_mut();
            let mut view = FixedPageView::<&mut [u8], R>::new_write(page_data.as_mut());

            log::info!("Header before insert");
            view.print_header();

            if let Some(slot_id) = view.insert(record) {
                log::info!("Header after insert");
                view.print_header();

                let has_free = view.has_free_slot();

                drop(view);
                drop(page_data);
                drop(page);

                if !has_free {
                    FreeSpaceManager::update_fixed(
                        self.bpm.clone(),
                        R::FSM_PAGE_DATA_TYPE,
                        free_page_idx,
                        false,
                    )?;
                }

                return Ok(SlottedRecordId::new(free_page_idx, slot_id));
            } else {
                // Если вставка не удалась, значит страница была заполнена другим потоком. Обновляем FSM и пробуем снова.

                drop(view);
                drop(page_data);
                drop(page);

                FreeSpaceManager::update_fixed(
                    self.bpm.clone(),
                    R::FSM_PAGE_DATA_TYPE,
                    free_page_idx,
                    false,
                )?;
            }
        }
    }

    pub fn get<R: FixedRecord>(&self, id: SlottedRecordId) -> Result<Option<R>, StorageError> {
        id.make_error_if_empty()?;
        let full_id = FullPageId {
            page_type: R::PAGE_DATA_TYPE,
            page_id: id.page_id(),
        };
        

        let page = self.bpm.get_page(full_id)?;
        let page_data = page.data();
        let view = FixedPageView::<&[u8], R>::new_read(page_data.as_ref());
        // view.print_header();
        let result = view.get(id.slot_id())?;

        Ok(result.cloned())
    }

    pub fn update<R: FixedRecord>(
        &self,
        id: SlottedRecordId,
        record: R,
    ) -> Result<(), StorageError> {
        id.make_error_if_empty()?;
        let full_id = FullPageId {
            page_type: R::PAGE_DATA_TYPE,
            page_id: id.page_id(),
        };

        let page = self.bpm.get_page(full_id)?;
        let mut page_data = page.data_mut();
        let mut view = FixedPageView::<&mut [u8], R>::new_write(page_data.as_mut());
        let value = view
            .get_mut(id.slot_id())?
            .ok_or(StorageError::Unexpected("".into()))?;
        *value = record;
        Ok(())
    }

    pub fn delete<R: FixedRecord>(&self, id: SlottedRecordId) -> Result<(), StorageError> {
        id.make_error_if_empty()?;
        let full_id = FullPageId {
            page_type: R::PAGE_DATA_TYPE,
            page_id: id.page_id(),
        };

        let page = self.bpm.get_page(full_id)?;
        let mut page_data = page.data_mut();
        let mut view = FixedPageView::<&mut [u8], R>::new_write(page_data.as_mut());
        let has_free_before = view.has_free_slot();
        view.delete(id.slot_id())?;

        if !has_free_before {
            drop(view);
            drop(page_data);
            drop(page);

            FreeSpaceManager::update_fixed(
                self.bpm.clone(),
                R::FSM_PAGE_DATA_TYPE,
                id.page_id(),
                true,
            )?;
        }
        Ok(())
    }

    pub fn iter<R: FixedRecord>(&self) -> FixedRecordIter<'_, R> {
        FixedRecordIter {
            manager: self,
            current_page: 0,
            current_slot: 0,
            _phantom: std::marker::PhantomData,
        }
    }

}

pub struct FixedRecordIter<'a, R> {
    manager: &'a FixedRecordManager,
    current_page: u32,
    current_slot: u16,
    _phantom: std::marker::PhantomData<R>, // Нужен, так как R используется только в сигнатурах
}

// 2. Реализуем Iterator для этой структуры
impl<'a, R: FixedRecord> Iterator for FixedRecordIter<'a, R> {
    type Item = (R, SlottedRecordId);

    fn next(&mut self) -> Option<Self::Item> {
        let page_count = self.manager.bpm.page_count(R::PAGE_DATA_TYPE).unwrap();

        loop {
            if self.current_page >= page_count {return None;}

            let page_id = FullPageId{ 
                   page_type: R::PAGE_DATA_TYPE, 
                   page_id: self.current_page 
            };
            let page = self.manager.bpm.get_page(page_id).unwrap();
            let page_data = page.data();
            let view = FixedPageView::<&[u8], R>::new_read(page_data.as_ref());

            loop {
                if let Ok(slot) = view.get(self.current_slot) {
                    match slot {
                        Some(s) => {
                            let id = SlottedRecordId::new(self.current_page, self.current_slot);
                            self.current_slot += 1;
                            return Some((*s, id))
                        },
                        None => {},
                    }
                    self.current_slot += 1;
                } else {
                    break;
                }
            }
            self.current_slot = 0;
            self.current_page += 1;
        }
    }
}

// impl IntoIterator for FixedRecordManager {
//     type Item;

//     type IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         todo!()
//     }
// }
