use parser::{EntityTypeDef, SetStmt};

use crate::{
    attribute::Value,
    query::{EngineError, QueryContext, QueryEngine, engine::Executable},
    storage::{AttributesRecord, EdgeRecord, MetaRecord, VertRecord},
};

impl Executable for SetStmt {
    fn execute(&self, qe: &QueryEngine, ctx: &mut QueryContext) -> Result<(), EngineError> {
        for part in &self.parts {
            let entity_ids = ctx.get_by_alias(&part.alias);
            if entity_ids.is_empty() {
                return Err(EngineError::UndefinedAlias(part.alias.clone()));
            }

            for e_id in entity_ids {
                let attr_id = match e_id.ty {
                    EntityTypeDef::Vertex => {
                        let vert: VertRecord = qe
                            .fixed_manager()
                            .get(e_id.id)?
                            .ok_or(EngineError::EntityNotFound)?;
                        vert.attr_record_id()
                    }
                    EntityTypeDef::Metavertex => {
                        let meta: MetaRecord = qe
                            .fixed_manager()
                            .get(e_id.id)?
                            .ok_or(EngineError::EntityNotFound)?;
                        meta.attr_record_id()
                    }
                    EntityTypeDef::Edge => {
                        let edge: EdgeRecord = qe
                            .fixed_manager()
                            .get(e_id.id)?
                            .ok_or(EngineError::EntityNotFound)?;
                        edge.attr_record_id()
                    }
                };

                if attr_id.is_empty() {
                    return Err(EngineError::AttributeNotExist(part.attribute_name.clone()));
                }

                let mut attr_record: AttributesRecord = qe
                    .slotted_manager()
                    .get(attr_id)?
                    .ok_or(EngineError::EntityNotFound)?;

                if let Some(attr) = attr_record.attrs.get_mut(&part.attribute_name) {
                    let new_value: Value = part.value.clone().into();
                    attr.set_value(new_value)?;

                    let new_attr_id = qe.slotted_manager().update(attr_id, attr_record)?;
                    if new_attr_id != attr_id {
                        match e_id.ty {
                            EntityTypeDef::Vertex => {
                                let mut vert: VertRecord = qe
                                    .fixed_manager()
                                    .get(e_id.id)?
                                    .ok_or(EngineError::EntityNotFound)?;
                                vert.set_attr_id(new_attr_id);
                                qe.fixed_manager().update(e_id.id, vert)?;
                            }
                            EntityTypeDef::Metavertex => {
                                let mut meta: MetaRecord = qe
                                    .fixed_manager()
                                    .get(e_id.id)?
                                    .ok_or(EngineError::EntityNotFound)?;
                                meta.set_attr_id(new_attr_id);
                                qe.fixed_manager().update(e_id.id, meta)?;
                            }
                            EntityTypeDef::Edge => {
                                let mut edge: EdgeRecord = qe
                                    .fixed_manager()
                                    .get(e_id.id)?
                                    .ok_or(EngineError::EntityNotFound)?;
                                edge.set_attr_id(new_attr_id);
                                qe.fixed_manager().update(e_id.id, edge)?;
                            }
                        };
                    }
                } else {
                    return Err(EngineError::AttributeNotExist(part.attribute_name.clone()));
                }
            }
        }

        Ok(())
    }
}
