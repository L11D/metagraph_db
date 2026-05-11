use std::{
    path::Path,
    sync::Arc,
};

use parser::Stmt;

use crate::{
    query::{common::{QueryContext, QueryResult}, error::EngineError},
    storage::{
        BufferPoolManager, DiskManager, FixedRecordManager, NamingManager, SlottedRecordManager
    },
};

pub struct QueryEngine {
    fixed_manager: FixedRecordManager,
    slotted_manager: SlottedRecordManager,
    naming_manager: NamingManager,
    bpm: Arc<BufferPoolManager>
}

impl QueryEngine {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        let disk_manager = Arc::new(DiskManager::new(path)?);
        let bpm = Arc::new(BufferPoolManager::new(disk_manager.clone(), 50)?);

        Ok(Self {
            fixed_manager: FixedRecordManager::new(bpm.clone()),
            slotted_manager: SlottedRecordManager::new(bpm.clone()),
            naming_manager: NamingManager::new(disk_manager),
            bpm: bpm
        })
    }

    pub fn execute(&self, stmt: Stmt) -> Result<QueryResult, EngineError>  {
        let mut ctx = QueryContext::new();
        stmt.execute(self, &mut ctx)?;
        Ok(ctx.result)
    }

    pub fn fixed_manager(&self) -> &FixedRecordManager {
        &self.fixed_manager
    }

    pub fn slotted_manager(&self) -> &SlottedRecordManager {
        &self.slotted_manager
    }

    pub fn naming_manager(&self) -> &NamingManager {
        &self.naming_manager
    }

    pub fn close(&self) {
        self.bpm.close();
    }

}

pub trait Executable {
    fn execute(&self, qe: &QueryEngine, ctx: &mut QueryContext) -> Result<(), EngineError>;
}

