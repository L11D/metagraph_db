use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;
use parking_lot::Mutex;

use crate::{
    bincode::Bincode,
    storage::{SlottedRecordId, disk_manager::DiskManager},
};

pub struct NamingManager {
    names: DashMap<String, SlottedRecordId>,
    disk_manager: Mutex<Arc<DiskManager>>,
}

impl NamingManager {
    pub fn new(disk_manager: Arc<DiskManager>) -> Self {
        let names_data = disk_manager.get_names();
        log::info!("Loaded names: {} bytes", names_data.len());
        let names: HashMap<String, SlottedRecordId> = if names_data.len() > 0 {
            Bincode::decode_from_slice(&names_data)
        } else {
            HashMap::new()
        };

        Self {
            names: DashMap::from_iter(names.into_iter()),
            disk_manager: Mutex::new(disk_manager),
        }
    }

    pub fn get(&self, name: &str) -> Option<SlottedRecordId> {
        self.names.get(name).map(|r| *r.value())
    }

    fn flush(&self) {
        let manager = self.disk_manager.lock();
        // Конвертируем DashMap обратно в HashMap/Vec для bincode
        let data: HashMap<String, SlottedRecordId> = self
            .names
            .iter()
            .map(|r| (r.key().clone(), *r.value()))
            .collect();

        let bytes = Bincode::encode_to_vec(&data);

        manager.save_names(&bytes);
    }

    pub fn insert(&self, name: String, id: SlottedRecordId) {
        self.names.insert(name, id);
        self.flush();
    }

    pub fn remove(&self, name: &str) {
        if self.names.remove(name).is_some() {
            self.flush();
        }
    }
}
