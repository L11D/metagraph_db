use std::collections::{BTreeSet, HashMap, HashSet};

use axum::response::sse::KeepAlive;
use parser::EntityTypeDef;
use serde::Serialize;

use crate::{query::error::EngineError, storage::SlottedRecordId};

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub header: Vec<String>,
    pub body: Vec<Vec<String>>,
}
impl QueryResult {
    pub fn new(header: Vec<String>, body: Vec<Vec<String>>) -> Self {
        Self { header, body }
    }
    pub fn new_empty() -> Self {
        Self {
            header: Vec::new(),
            body: Vec::new(),
        }
    }
}

pub struct QueryContext {
    pub rows: HashMap<BTreeSet<String>, Vec<HashMap<String, FullEntityId>>>,
    pub result: QueryResult,
    // pub curr_stmt: i32
}

impl QueryContext {
    pub fn new() -> Self {
        return Self {
            rows: HashMap::new(),
            result: QueryResult::new_empty(),
        };
    }

    pub fn add_entity(&mut self, alias: &str, id: FullEntityId) {
        let key = BTreeSet::from_iter(vec![alias.to_string()]);
        let mut row: HashMap<String, FullEntityId> = HashMap::new();
        row.insert(alias.to_string(), id);
        self.add_row(&key, row);
    }

    pub fn add_row(&mut self, aliases: &BTreeSet<String>, row: HashMap<String, FullEntityId>) {
        let stmt_row = self.rows.entry(aliases.clone()).or_insert(Vec::new());
        stmt_row.push(row);
    }

    pub fn delete_rows(&mut self, aliases: &BTreeSet<String>) {
        self.rows.remove(aliases);
    }

    pub fn get_rows(&self, aliases: &BTreeSet<String>) -> &Vec<HashMap<String, FullEntityId>> {
        self.rows.get(aliases).unwrap()
    }

    pub fn set_rows(
        &mut self,
        aliases: &BTreeSet<String>,
        rows: Vec<HashMap<String, FullEntityId>>,
    ) {
        let stmt_row = self.rows.entry(aliases.clone()).or_insert(Vec::new());
        *stmt_row = rows;
    }

    pub fn get_by_alias(&self, alias: &str) -> Vec<FullEntityId> {
        let _key = BTreeSet::from_iter(vec![alias.to_string()]);

        let key = self.rows.keys().find(|k| _key.is_subset(k));
        let mut result = Vec::new();

        match key {
            Some(k) => {
                let rows = self.rows.get(k).unwrap();
                for row in rows {
                    if let Some(id) = row.get(alias) {
                        result.push(id.clone());
                    }
                }
            }
            None => {},
        }

        result
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub struct FullEntityId {
    pub id: SlottedRecordId,
    pub ty: EntityTypeDef,
}
impl FullEntityId {
    pub fn new(id: SlottedRecordId, ty: EntityTypeDef) -> Self {
        Self { id, ty }
    }

    pub fn to_string(&self) -> String {
        let ty = match self.ty {
            EntityTypeDef::Vertex => "V",
            EntityTypeDef::Edge => "E",
            EntityTypeDef::Metavertex => "M",
        };

        format!("{}:{}", ty, self.id.human_readable())
    }

    pub fn from_string(str: &str) -> Result<Self, EngineError> {
        let parts = str.split(':').collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(EngineError::Unexpected(format!(
                "Invalid entity id format: {str}"
            )));
        }
        let ty = match parts[0] {
            "V" => EntityTypeDef::Vertex,
            "E" => EntityTypeDef::Edge,
            "M" => EntityTypeDef::Metavertex,
            _ => {
                return Err(EngineError::Unexpected(format!(
                    "Invalid entity type: {}",
                    parts[0]
                )));
            }
        };
        let id_str = parts[1];
        let id = SlottedRecordId::from_string(id_str)?;
        Ok(Self { id, ty })
    }
}
