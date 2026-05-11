use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc},
};

use parking_lot::Mutex;

use crate::query::{EngineError, QueryEngine, QueryResult};

pub struct Database {
    engine: QueryEngine,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Self {
        Self {
            engine: QueryEngine::new(path).unwrap(),
        }
    }

    pub fn execute(&self, query: &str) -> Result<QueryResult, EngineError> {
        let stmt = parser::parse(query)?;
        log::info!("AST: {:#?}", stmt);
        let result = self.engine.execute(stmt)?;
        log::info!("result: {:#?}", result);
        Ok(result)
    }

    pub fn close(&self) {
        self.engine.close();
    }
}

pub struct DatabaseManager {
    main_dir: PathBuf,
    open_databases: Mutex<HashMap<String, Arc<Database>>>,
}

impl DatabaseManager {
    pub fn new(path: impl AsRef<Path>) -> Self {
        fs::create_dir_all(path.as_ref()).unwrap();
        Self {
            main_dir: path.as_ref().to_path_buf(),
            open_databases: Mutex::new(HashMap::new()),
        }
    }

    pub fn open(&self, name: &str) -> Arc<Database> {
        let mut open_databases = self.open_databases.lock();
        if let Some(db) = open_databases.get(name) {
            return db.clone();
        }
        let path = self.main_dir.join(name);
        let db = Arc::new(Database::open(path));
        open_databases.insert(name.to_string(), db.clone());
        db
    }

    pub fn close(&self) {
        let lock = self.open_databases.lock();
        for (_, database) in &*lock {
            database.close()
        }
    }
}
