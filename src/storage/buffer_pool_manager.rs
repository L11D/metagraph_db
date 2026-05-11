use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst}},
};

use parking_lot::{Condvar, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::storage::{
    LruKReplacer, StorageError,
    common::{PAGE_SIZE, PageDataType},
    disk_manager::DiskManager,
};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct FullPageId {
    pub page_type: PageDataType,
    pub page_id: u32,
}

struct BpmState {
    page_table: HashMap<FullPageId, usize>,
    free_list: Vec<usize>,
    replacer: LruKReplacer,
    frame_to_page: Vec<Option<FullPageId>>,

    // Множество страниц, которые прямо сейчас читаются или пишутся на диск
    io_in_progress: HashSet<FullPageId>,
}

pub struct BufferPoolManager {
    disk_manager: Arc<DiskManager>,
    pages: Vec<Page>,

    // Единый мьютекс и условная переменная для синхронизации
    state: Mutex<BpmState>,
    cvar: Condvar,
}

impl BufferPoolManager {
    pub fn new(disk_manager: Arc<DiskManager>, pool_size: usize) -> Result<Self, StorageError> {
        let mut pages = Vec::with_capacity(pool_size);
        let mut free_list = Vec::with_capacity(pool_size);
        let mut frame_to_page = Vec::with_capacity(pool_size);

        for i in 0..pool_size {
            pages.push(Page::new(i as u32, [0; 4096]));
            free_list.push(i);
            frame_to_page.push(None);
        }

        let state = BpmState {
            page_table: HashMap::new(),
            free_list,
            replacer: LruKReplacer::new(2),
            frame_to_page,
            io_in_progress: HashSet::new(),
        };

        Ok(Self {
            disk_manager,
            pages,
            state: Mutex::new(state),
            cvar: Condvar::new(),
        })
    }

    pub fn get_page(&self, page_id: FullPageId) -> Result<PageGuard<'_>, StorageError> {
        let mut state = self.state.lock();

        loop {
            // 1. Избегаем Double Fetch и I/O Race: ждем, если страница в процессе ввода-вывода
            if state.io_in_progress.contains(&page_id) {
                self.cvar.wait(&mut state);
                continue; // После пробуждения начинаем всё сначала
            }

            // 2. Страница уже в памяти
            if let Some(&frame_id) = state.page_table.get(&page_id) {
                let page = &self.pages[frame_id];
                page.pin(); // Пиним ДО отпускания мьютекса! (решает Eviction Race)
                state.replacer.record_access(frame_id);
                return Ok(PageGuard::new_already_pinned(page));
            }

            // 3. Страницы нет в памяти, нужно аллоцировать фрейм
            let frame_id = if let Some(id) = state.free_list.pop() {
                id
            } else {
                state.replacer.evict(&self.pages)? // Evict проверяет is_pinned
            };

            let page = &self.pages[frame_id];
            let old_page_id_opt = state.frame_to_page[frame_id];

            // 4. Очищаем метаданные старой страницы (если она была)
            if let Some(old_page_id) = old_page_id_opt {
                if page.is_dirty() {
                    // Блокируем доступ к старой странице, пока мы пишем её на диск
                    state.io_in_progress.insert(old_page_id);
                }
                state.page_table.remove(&old_page_id);
            }

            // 5. Бронируем фрейм под новую страницу ДО начала I/O
            state.io_in_progress.insert(page_id);
            state.frame_to_page[frame_id] = Some(page_id);
            state.page_table.insert(page_id, frame_id);
            page.pin();
            state.replacer.record_access(frame_id);

            // 6. Отпускаем блокировку метаданных. I/O не должно блокировать другие потоки!
            drop(state);

            // Получаем эксклюзивный доступ к данным страницы
            let mut data_guard = page.data_write_internal();

            // 7. Сбрасываем старую страницу на диск
            if let Some(old_page_id) = old_page_id_opt {
                if page.is_dirty() {
                    self.disk_manager.write_page(old_page_id, **data_guard)?;
                    page.unmark_dirty();

                    // Сообщаем, что запись старой страницы завершена
                    let mut state = self.state.lock();
                    state.io_in_progress.remove(&old_page_id);
                    self.cvar.notify_all();
                    drop(state);
                }
            }

            // 8. Читаем новые данные
            let new_data = self.disk_manager.read_page(page_id)?;
            *data_guard = Box::new(new_data);
            drop(data_guard); // Отпускаем блокировку данных

            // 9. Сообщаем, что загрузка завершена (будим потоки, ждавшие Double Fetch)
            let mut state = self.state.lock();
            state.io_in_progress.remove(&page_id);
            self.cvar.notify_all();

            return Ok(PageGuard::new_already_pinned(page));
        }
    }

    pub fn page_count(&self, page_type: PageDataType) -> Result<u32, StorageError> {
        self.disk_manager.page_count(page_type)
    }

    pub fn close(&self) {
        log::info!("Closing bpm");
        let state = self.state.lock();
        for (id, n ) in &state.page_table {
            let page = &self.pages[*n];
            let data_guard = page.data_write_internal();
            if page.is_dirty() {
                let _ = self.disk_manager.write_page(*id, **data_guard);
            }
        }
    }

}

// --- 2. Изменения в PageGuard ---
pub struct PageGuard<'a> {
    page: &'a Page,
}

impl<'a> PageGuard<'a> {
    /// Страница УЖЕ должна быть закреплена перед созданием Guard
    pub(crate) fn new_already_pinned(page: &'a Page) -> Self {
        Self { page }
    }
}

impl Drop for PageGuard<'_> {
    fn drop(&mut self) {
        self.page.unpin();
    }
}

impl Deref for PageGuard<'_> {
    type Target = Page;
    fn deref(&self) -> &Self::Target {
        self.page
    }
}

pub struct Page {
    id: u32,
    data: RwLock<Box<[u8; PAGE_SIZE]>>,
    is_dirty: AtomicBool,
    pin_count: AtomicUsize,
}

impl Page {
    pub fn new(id: u32, data: [u8; PAGE_SIZE]) -> Self {
        Page {
            id: id,
            data: RwLock::new(Box::new(data)),
            is_dirty: AtomicBool::new(false),
            pin_count: AtomicUsize::new(0),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub(crate) fn data_write_internal(&self) -> RwLockWriteGuard<'_, Box<[u8; PAGE_SIZE]>> {
        self.data.write()
    }

    // pub fn swap_data(&self, new_data: [u8; PAGE_SIZE]) -> [u8; PAGE_SIZE] {
    //     let mut data_guard = self.data.write();
    //     let old_data = std::mem::replace(&mut *data_guard, Box::new(new_data));
    //     self.unmark_dirty();
    //     self.drop_pin();
    //     *old_data
    // }

    pub fn data(&self) -> RwLockReadGuard<'_, Box<[u8; PAGE_SIZE]>> {
        self.data.read()
    }

    pub fn data_mut(&self) -> RwLockWriteGuard<'_, Box<[u8; PAGE_SIZE]>> {
        self.mark_dirty();
        self.data.write()
    }

    fn mark_dirty(&self) {
        self.is_dirty.store(true, SeqCst);
    }

    fn unmark_dirty(&self) {
        self.is_dirty.store(false, SeqCst);
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty.load(SeqCst)
    }

    fn pin(&self) {
        self.pin_count.fetch_add(1, SeqCst);
    }

    fn unpin(&self) {
        self.pin_count.fetch_sub(1, SeqCst);
    }

    // fn drop_pin(&self) {
    //     self.pin_count.store(0, SeqCst);
    // }

    pub fn is_pinned(&self) -> bool {
        self.pin_count.load(SeqCst) as u32 > 0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};


    use super::*;

    // #[test]
    // fn test() {
    //     let buffer_pool_manager = BufferPoolManager::new("data", 5).unwrap();
    //     let page_id = FullPageId {
    //         page_type: PageDataType::Vert,
    //         page_id: 0,
    //     };

    //     let page = buffer_pool_manager.get_page(page_id).unwrap();

    //     let mut binding = page.data_mut();
    //     // let data = binding.as_mut();

    //     // let data = page.data();

    //     let mut view = FixedPageView::<_, VertexRecord>::new_write(binding.as_mut());

    //     let id = view.insert(VertexRecord::default()).unwrap();

    //     let v = view.get_mut(id).unwrap().unwrap();
    //     v.parent = 42;
    // }

    // #[test]
    // fn test2() {
    //     let buffer_pool_manager = BufferPoolManager::new("data", 5).unwrap();
    //     let page_id = FullPageId {
    //         page_type: PageDataType::Attribute,
    //         page_id: 0,
    //     };

    //     let page = buffer_pool_manager.get_page(page_id).unwrap();

    //     let mut binding = page.data_mut();
    //     // let data = binding.as_mut();

    //     // let data = page.data();

    //     let mut view = SlottedPageView::<_, VertexStor>::new_write(binding.as_mut());

    //     let ver = VertexStor {
    //         attributes: HashMap::new(),
    //         parents: HashSet::new(),
    //         edges: HashSet::new(),
    //     };

    //     let id = view.insert(&ver).unwrap();

    //     let v = view.get(id).unwrap();
    //     println!("Vertex: {:?}", v);
    // }
}
