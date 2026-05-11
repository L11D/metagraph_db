use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    os::unix::fs::FileExt,
    path::Path,
};

use crate::{bincode::Bincode, storage::{
    StorageError,
    buffer_pool_manager::FullPageId,
    common::{PAGE_SIZE, PageDataType}, page_view
}};

pub struct DiskManager {
    files: HashMap<PageDataType, File>,
    names_file: File
}

impl DiskManager {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        fs::create_dir_all(path.as_ref()).unwrap();
        let mut files = HashMap::new();
        for &page_type in PageDataType::ALL {
            let file_name = Self::get_file_name(page_type);
            let file_path = path.as_ref().join(file_name);
            let file = Self::open_file(file_path)?;
            files.insert(page_type, file);
        }
        let names_file_path = path.as_ref().join("names.dat");
        let names_file = Self::open_file(names_file_path)?;
        // Инициализация диска и других необходимых компонентов
        Ok(Self { files, names_file })
    }

    pub fn read_page(&self, id: FullPageId) -> Result<[u8; PAGE_SIZE], StorageError> {
        let mut buf = [0; PAGE_SIZE];

        if PageDataType::SLOTTED.contains(&id.page_type) {
            page_view::init_header(&mut buf);
        }

        let file = self
            .files
            .get(&id.page_type)
            .ok_or(StorageError::Unexpected("".into()))?;

        let size = file.metadata().unwrap().len();

        let start = id.page_id as u64 * PAGE_SIZE as u64;
        let end = start + PAGE_SIZE as u64;

        if size < end {
            file.set_len(size + end).unwrap();
            return Ok(buf);
        }

        file.read_at(&mut buf, start)
            .map_err(|_| StorageError::Unexpected("".into()))?;

        Ok(buf)
    }

    pub fn write_page(&self, id: FullPageId, data: [u8; PAGE_SIZE]) -> Result<(), StorageError> {
        let offset = id.page_id as u64 * PAGE_SIZE as u64;

        let file = self
            .files
            .get(&id.page_type)
            .ok_or(StorageError::Unexpected("".into()))?;

        let size = file.metadata().unwrap().len();

        let new_size = offset + PAGE_SIZE as u64;

        if size < new_size {
            file.set_len(size + new_size).unwrap();
        }

        file.write_at(&data, offset)
            .map_err(|_| StorageError::Unexpected("".into()))?;

        Ok(())
    }

    pub fn page_count(&self, page_type: PageDataType) -> Result<u32, StorageError> {
        let file = self
            .files
            .get(&page_type)
            .ok_or(StorageError::Unexpected("".into()))?;

        let size = file.metadata().unwrap().len();
        Ok((size / PAGE_SIZE as u64) as u32)
    }

    pub fn get_names(&self) -> Vec<u8> {
        let mut size_buf = [0; 8];
        self.names_file.read_at(&mut size_buf, 0).unwrap();
        let names_size = usize::from_le_bytes(size_buf);

        let mut result = vec![0u8; names_size];
        self.names_file.read_at(&mut result, 8).unwrap();
        result
    }

    pub fn save_names(&self, names: &[u8]) {
        self.names_file.write_at(&names.len().to_le_bytes(), 0).unwrap();
        self.names_file.write_at(names, 8).unwrap();
    }

    fn get_file_name(page_type: PageDataType) -> &'static str {
        match page_type {
            PageDataType::Attribute => "attribute.dat",
            PageDataType::Parents => "parents.dat",
            PageDataType::MetaInfo => "meta_info.dat",
            PageDataType::Vert => "vert.dat",
            PageDataType::Meta => "meta.dat",
            PageDataType::Edge => "edge.dat",
            
            PageDataType::AttributeFsm => "attribute.fsm",
            PageDataType::ParentsFsm => "parents.fsm",
            PageDataType::MetaInfoFsm => "meta_info.fsm",
            PageDataType::VertFsm => "vert.fsm",
            PageDataType::MetaFsm => "meta.fsm",
            PageDataType::EdgeFsm => "edge.fsm",
        }
    }

    fn open_file(path: impl AsRef<Path>) -> Result<File, StorageError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|e| StorageError::Unexpected(format!("Failed to create file: {}", e)))?;
        Ok(file)
    }
}
