use std::collections::{HashMap, HashSet};

use parser::{EntityTypeDef, ParentStmt};

use crate::{
    attribute::Attribute,
    metagraph::ChildConstraint,
    query::{EngineError, QueryContext, QueryEngine, engine::Executable},
    storage::{AttributesRecord, MetaInfoRecord, MetaRecord, SlottedRecordId, VertRecord},
};

impl Executable for ParentStmt {
    fn execute(&self, qe: &QueryEngine, ctx: &mut QueryContext) -> Result<(), EngineError> {
        let parent_ids = ctx.get_by_alias(&self.parent);
        if parent_ids.is_empty() {
            return Err(EngineError::UndefinedAlias(self.parent.clone()));
        }

        let child_ids = ctx.get_by_alias(&self.child);
        if child_ids.is_empty() {
            return Err(EngineError::UndefinedAlias(self.child.clone()));
        }

        if !self.is_parenting {
            for id in &parent_ids {
                if !matches!(id.ty, EntityTypeDef::Metavertex) {
                    return Err(EngineError::ParentToNotMeta);
                }
                let mut meta: MetaRecord = qe
                    .fixed_manager()
                    .get(id.id)?
                    .ok_or(EngineError::EntityNotFound)?;

                if meta.meta_info_id().is_empty() {
                    continue;
                }

                let v_children: HashSet<SlottedRecordId> = child_ids
                    .iter()
                    .filter(|id| matches!(id.ty, EntityTypeDef::Vertex))
                    .map(|id| id.id)
                    .collect();

                let mv_children: HashSet<SlottedRecordId> = child_ids
                    .iter()
                    .filter(|id| matches!(id.ty, EntityTypeDef::Metavertex))
                    .map(|id| id.id)
                    .collect();

                let mut meta_info: MetaInfoRecord = qe
                    .slotted_manager()
                    .get(meta.meta_info_id())?
                    .ok_or(EngineError::EntityNotFound)?;
                meta_info.remove_v_children(v_children);
                meta_info.remove_mv_children(mv_children);
                let new_id = qe
                    .slotted_manager()
                    .update(meta.meta_info_id(), meta_info)?;
                if new_id != meta.meta_info_id() {
                    meta.set_meta_info_id(new_id);
                    qe.fixed_manager().update(id.id, meta)?;
                }
            }

            return Ok(());
        }

        let mut child_vs = HashMap::new();
        let mut child_mvs = HashMap::new();

        for id in child_ids {
            match id.ty {
                EntityTypeDef::Vertex => {
                    let vert: VertRecord = qe
                        .fixed_manager()
                        .get(id.id)?
                        .ok_or(EngineError::EntityNotFound)?;
                    child_vs.insert(id.id, vert);
                }
                EntityTypeDef::Metavertex => {
                    let meta: MetaRecord = qe
                        .fixed_manager()
                        .get(id.id)?
                        .ok_or(EngineError::EntityNotFound)?;
                    child_mvs.insert(id.id, meta);
                }
                EntityTypeDef::Edge => todo!(),
            }
        }

        for id in parent_ids {
            if !matches!(id.ty, EntityTypeDef::Metavertex) {
                return Err(EngineError::ParentToNotMeta);
            }
            let mut meta: MetaRecord = qe
                .fixed_manager()
                .get(id.id)?
                .ok_or(EngineError::EntityNotFound)?;

            if meta.meta_info_id().is_empty() {
                let v_children: HashSet<SlottedRecordId> = child_vs.keys().cloned().collect();
                let mv_children: HashSet<SlottedRecordId> = child_mvs.keys().cloned().collect();
                let meta_info = MetaInfoRecord::new(v_children, mv_children, HashSet::new());
                let meta_info_id = qe.slotted_manager().insert(meta_info)?;
                meta.set_meta_info_id(meta_info_id);
                qe.fixed_manager().update(id.id, meta)?;
            } else {
                let mut meta_info: MetaInfoRecord = qe
                    .slotted_manager()
                    .get(meta.meta_info_id())?
                    .ok_or(EngineError::EntityNotFound)?;
                for constr in &meta_info.constraints {
                    for (v_id, vert) in &child_vs {
                        let attrs: HashMap<String, Attribute> = if vert.attr_record_id().is_empty()
                        {
                            HashMap::new()
                        } else {
                            let attr_record: AttributesRecord = qe
                                .slotted_manager()
                                .get(vert.attr_record_id())?
                                .ok_or(EngineError::EntityNotFound)?;
                            attr_record.attrs
                        };
                        constr.check(&attrs)?;
                    }

                    for (v_id, meta) in &child_mvs {
                        let attrs: HashMap<String, Attribute> = if meta.attr_record_id().is_empty()
                        {
                            HashMap::new()
                        } else {
                            let attr_record: AttributesRecord = qe
                                .slotted_manager()
                                .get(meta.attr_record_id())?
                                .ok_or(EngineError::EntityNotFound)?;
                            attr_record.attrs
                        };

                        let constrains: HashSet<ChildConstraint> = if meta.meta_info_id().is_empty()
                        {
                            HashSet::new()
                        } else {
                            let meta_info: MetaInfoRecord = qe
                                .slotted_manager()
                                .get(meta.meta_info_id())?
                                .ok_or(EngineError::EntityNotFound)?;
                            meta_info.constraints
                        };
                        if !constrains.contains(&constr) {
                            constr.check(&attrs)?;
                        }
                    }
                }
                let v_children: HashSet<SlottedRecordId> = child_vs.keys().cloned().collect();
                let mv_children: HashSet<SlottedRecordId> = child_mvs.keys().cloned().collect();
                meta_info.add_v_children(v_children);
                meta_info.add_mv_children(mv_children);
                let new_id = qe
                    .slotted_manager()
                    .update(meta.meta_info_id(), meta_info)?;
                if new_id != meta.meta_info_id() {
                    meta.set_meta_info_id(new_id);
                    qe.fixed_manager().update(id.id, meta)?;
                }
            }
        }

        Ok(())
    }
}
