use std::sync::Arc;

use bincode::{Decode, Encode};

use crate::{
    bincode::Bincode,
    storage::{
        SlottedRecordId, StorageError, buffer_pool_manager::{BufferPoolManager, FullPageId}, common::PageDataType, free_space_manager::FreeSpaceManager, page_view::{self, SlottedPageView}
    },
};

pub trait SlottedRecord: Encode + Decode<()> {
    const PAGE_DATA_TYPE: PageDataType;
    const FSM_PAGE_DATA_TYPE: PageDataType;
}

pub struct SlottedRecordManager {
    bpm: Arc<BufferPoolManager>,
}
impl SlottedRecordManager {
    pub fn new(bpm: Arc<BufferPoolManager>) -> Self {
        Self { bpm }
    }

    pub fn insert<R: SlottedRecord>(&self, record: R) -> Result<SlottedRecordId, StorageError> {
        loop {
            let data = Bincode::encode_to_vec(&record);
            let record_size = data.len();
            if record_size > page_view::MAX_RECORD_SIZE {
                return Err(StorageError::OverflowRecord);
            }

            let free_page_idx = FreeSpaceManager::find_free_slotted_page(
                self.bpm.clone(),
                R::FSM_PAGE_DATA_TYPE,
                record_size,
            )?;

            let full_id = FullPageId {
                page_type: R::PAGE_DATA_TYPE,
                page_id: free_page_idx,
            };

            let page = self.bpm.get_page(full_id)?;
            let mut page_data = page.data_mut();
            let mut view = SlottedPageView::<&mut [u8], R>::new_write(page_data.as_mut());

            if let Ok(slot_id) = view.insert(&data) {
                let free_space = view.free_space();

                drop(view);
                drop(page_data);
                drop(page);

                FreeSpaceManager::update_slotted(
                    self.bpm.clone(),
                    R::FSM_PAGE_DATA_TYPE,
                    free_page_idx,
                    free_space,
                )?;

                return Ok(SlottedRecordId::new(free_page_idx, slot_id));
            } else {
                // Если вставка не удалась, значит страница была заполнена другим потоком.
                // Обновляем FSM и пробуем снова.
                let free_space = view.free_space();
                drop(view);
                drop(page_data);
                drop(page);

                FreeSpaceManager::update_slotted(
                    self.bpm.clone(),
                    R::FSM_PAGE_DATA_TYPE,
                    free_page_idx,
                    free_space,
                )?;
            }
        }
    }

    pub fn get<R: SlottedRecord>(&self, id: SlottedRecordId) -> Result<Option<R>, StorageError> {
        id.make_error_if_empty()?;
        let full_id = FullPageId {
            page_type: R::PAGE_DATA_TYPE,
            page_id: id.page_id(),
        };

        let page = self.bpm.get_page(full_id)?;
        let page_data = page.data();
        let view = SlottedPageView::<&[u8], R>::new_read(page_data.as_ref());

        if let Some(record) = view.get(id.slot_id())? {
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    pub fn update<R: SlottedRecord>(
        &self,
        id: SlottedRecordId,
        record: R,
    ) -> Result<SlottedRecordId, StorageError> {
        id.make_error_if_empty()?;
        let full_id = FullPageId {
            page_type: R::PAGE_DATA_TYPE,
            page_id: id.page_id(),
        };

        let page = self.bpm.get_page(full_id)?;
        let mut page_data = page.data_mut();
        let mut view = SlottedPageView::<&mut [u8], R>::new_write(page_data.as_mut());

        match view.update(id.slot_id(), &record) {
            Ok(new_slot_id) => {
                if new_slot_id != id.slot_id() {
                    // Если запись была перемещена, нужно обновить FSM
                    let free_space = view.free_space();
                    drop(view);
                    drop(page_data);
                    drop(page);

                    FreeSpaceManager::update_slotted(
                        self.bpm.clone(),
                        R::FSM_PAGE_DATA_TYPE,
                        id.page_id(),
                        free_space,
                    )?;
                }
                return Ok(SlottedRecordId::new(id.page_id(), new_slot_id));
            }

            Err(StorageError::PageFull) => {
                // Недостаточно места для обновления записи
                return self.insert(record)
            },

            Err(e) => return Err(e)
        }
    }

    pub fn delete<R: SlottedRecord>(&self, id: SlottedRecordId) -> Result<(), StorageError> {
        id.make_error_if_empty()?;
        let full_id = FullPageId {
            page_type: R::PAGE_DATA_TYPE,
            page_id: id.page_id(),
        };

        let page = self.bpm.get_page(full_id)?;
        let mut page_data = page.data_mut();
        let mut view = SlottedPageView::<&mut [u8], R>::new_write(page_data.as_mut());

        view.delete(id.slot_id())?;

        Ok(())
    }

}
