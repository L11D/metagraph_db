use std::collections::{BTreeSet, HashMap, HashSet};

use parser::{EntityTypeDef, ReturnStmt};

use crate::{
    query::{
        FullEntityId, QueryEngine,
        common::{QueryContext, QueryResult},
        engine::Executable,
        error::EngineError,
    },
    storage::{AttributesRecord, EdgeRecord, MetaRecord, VertRecord},
};
fn format_attr_name(alias: &str, attr_name: &str) -> String {
    format!("{} ({})", attr_name, alias)
}
impl Executable for ReturnStmt {
    fn execute(&self, qe: &QueryEngine, ctx: &mut QueryContext) -> Result<(), EngineError> {
        let aliases: BTreeSet<String> = self.parts.iter().map(|part| part.alias.clone()).collect();
        let mut attr_names: HashMap<String, HashSet<String>> = HashMap::new();

        for part in &self.parts {
            let names = attr_names
                .entry(part.alias.clone())
                .or_insert(HashSet::new());
            if let Some(name) = &part.attribute_name {
                names.insert(name.clone());
            }
        }

        log::info!("ctx: {:#?}", ctx.rows);

        let key = ctx.rows.keys().find(|key| aliases.is_subset(key));
        let key = match key {
            Some(k) => k,
            None => {
                return Ok(());
            }
        };


        let rows = ctx.rows.get(key).unwrap();
        let mut attr_pool: HashMap<FullEntityId, AttributesRecord> = HashMap::new();
        for row in rows {
            for id in row.values() {
                let attr_id = match id.ty {
                    EntityTypeDef::Vertex => {
                        let vert = qe.fixed_manager().get::<VertRecord>(id.id)?.unwrap();
                        vert.attr_record_id()
                    }
                    EntityTypeDef::Metavertex => {
                        let meta = qe.fixed_manager().get::<MetaRecord>(id.id)?.unwrap();
                        meta.attr_record_id()
                    }
                    EntityTypeDef::Edge => {
                        let edge = qe.fixed_manager().get::<EdgeRecord>(id.id)?.unwrap();
                        edge.attr_record_id()
                    }
                };

                if !attr_id.is_empty() {
                    let attr_record = qe.slotted_manager().get(attr_id)?.unwrap();
                    attr_pool.insert(id.clone(), attr_record);
                } else {
                    attr_pool.insert(id.clone(), AttributesRecord::new_empty());
                }
            }
        }

        let mut header = Vec::new();
        let mut row_maps: Vec<HashMap<String, String>> = Vec::new();

        for row in rows {
            let mut row_map: HashMap<String, String> = HashMap::new();
            for (alias, id) in row {
                if !aliases.contains(alias) {
                    continue;
                }

                let a_names = attr_names.get(alias).unwrap();

                if a_names.contains("id") || a_names.is_empty() {
                    let col_id = format_attr_name(alias, "id");
                    if !header.contains(&col_id) {
                        header.push(col_id.clone());
                    }
                    row_map.insert(col_id, id.to_string());
                }

                if let Some(attr_record) = attr_pool.get(id) {
                    for (attr_name, attr) in &attr_record.attrs {
                        if a_names.contains(attr_name) || a_names.is_empty() {
                            let col_name = format_attr_name(alias, attr_name);
                            if !header.contains(&col_name) {
                                header.push(col_name.clone());
                            }
                            row_map.insert(col_name, attr.value().to_string());
                        }
                    }
                }
            }
            row_maps.push(row_map);
        }

        let mut result_body: Vec<Vec<String>> = Vec::with_capacity(row_maps.len());

        for mut row in row_maps {
            let mut table_row = Vec::with_capacity(header.len());

            // Проходим строго по порядку колонок в сформированной шапке
            for col_name in &header {
                if let Some(val) = row.remove(col_name) {
                    table_row.push(val);
                } else {
                    // Если у записи нет атрибута, который есть у других, ставим "undefined"
                    table_row.push("Undefined".to_string());
                }
            }
            result_body.push(table_row);
        }

        ctx.result = QueryResult::new(header, result_body);
        Ok(())
    }
}
