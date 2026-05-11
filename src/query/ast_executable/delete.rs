use parser::{DeleteStmt, EntityTypeDef};

use crate::{
    query::{EngineError, QueryContext, QueryEngine, engine::Executable},
    storage::{
        AttributesRecord, EdgeDestination, EdgeRecord, MetaInfoRecord, MetaRecord, ParentsRecord, SlottedRecordId, VertRecord
    },
};

impl Executable for DeleteStmt {
    fn execute(&self, qe: &QueryEngine, ctx: &mut QueryContext) -> Result<(), EngineError> {
        for part in &self.parts {
            let entity_ids = ctx.get_by_alias(&part.alias);
            if entity_ids.is_empty() {
                return Err(EngineError::UndefinedAlias(part.alias.clone()));
            }

            for e_id in entity_ids {
                match e_id.ty {
                    EntityTypeDef::Vertex => {
                        let vert: VertRecord = qe
                            .fixed_manager()
                            .get(e_id.id)?
                            .ok_or(EngineError::EntityNotFound)?;
                        if !vert.attr_record_id().is_empty() {
                            qe.slotted_manager()
                                .delete::<AttributesRecord>(vert.attr_record_id())?;
                        }
                        if !vert.next_edge_id().is_empty() {
                            let mut edge_id = vert.next_edge_id();
                            loop {
                                let edge: EdgeRecord = qe
                                    .fixed_manager()
                                    .get(edge_id)?
                                    .ok_or(EngineError::EntityNotFound)?;

                                if !edge.attr_record_id().is_empty() {
                                    qe.slotted_manager()
                                        .delete::<AttributesRecord>(edge.attr_record_id())?;
                                }

                                let next_edge_id = if edge.from().id() == e_id.id {
                                    edge.from_next_edge_id()
                                } else if edge.to().id() == e_id.id {
                                    edge.to_next_edge_id()
                                } else {
                                    panic!("Node is not part of this edge");
                                };

                                qe.fixed_manager().delete::<EdgeRecord>(edge_id)?;
                                if next_edge_id.is_empty() {
                                    break;
                                } else {
                                    edge_id = next_edge_id;
                                }
                            }
                        }
                        if vert.parent().is_link() {
                            qe.slotted_manager()
                                .delete::<ParentsRecord>(vert.parent())?;
                        }
                        qe.fixed_manager().delete::<VertRecord>(e_id.id)?;
                    }

                    EntityTypeDef::Metavertex => {
                        let meta: MetaRecord = qe
                            .fixed_manager()
                            .get(e_id.id)?
                            .ok_or(EngineError::EntityNotFound)?;
                        if !meta.attr_record_id().is_empty() {
                            qe.slotted_manager()
                                .delete::<AttributesRecord>(meta.attr_record_id())?;
                        }
                        if !meta.next_edge_id().is_empty() {
                            let mut edge_id = meta.next_edge_id();
                            loop {
                                let edge: EdgeRecord = qe
                                    .fixed_manager()
                                    .get(edge_id)?
                                    .ok_or(EngineError::EntityNotFound)?;

                                if !edge.attr_record_id().is_empty() {
                                    qe.slotted_manager()
                                        .delete::<AttributesRecord>(edge.attr_record_id())?;
                                }

                                let next_edge_id = if edge.from().id() == e_id.id {
                                    edge.from_next_edge_id()
                                } else if edge.to().id() == e_id.id {
                                    edge.to_next_edge_id()
                                } else {
                                    panic!("Node is not part of this edge");
                                };

                                qe.fixed_manager().delete::<EdgeRecord>(edge_id)?;
                                if next_edge_id.is_empty() {
                                    break;
                                } else {
                                    edge_id = next_edge_id;
                                }
                            }
                        }
                        if meta.parent().is_link() {
                            qe.slotted_manager()
                                .delete::<ParentsRecord>(meta.parent())?;
                        }
                        if !meta.meta_info_id().is_empty() {
                            qe.slotted_manager()
                                .delete::<MetaInfoRecord>(meta.meta_info_id())?;
                        }
                        qe.fixed_manager().delete::<MetaRecord>(e_id.id)?;
                    }

                    EntityTypeDef::Edge => {
                        // let edge: EdgeRecord = qe
                        //     .fixed_manager()
                        //     .get(e_id.id)?
                        //     .ok_or(EngineError::EntityNotFound)?;
                        // qe.fixed_manager().delete::<EdgeRecord>(e_id.id)?;

                        // if !edge.attr_record_id().is_empty() {
                        //     qe.slotted_manager()
                        //         .delete::<AttributesRecord>(edge.attr_record_id())?;
                        // }

                        // match edge.from() {
                        //     EdgeDestination::Vert(id) => {
                        //         let mut vert: VertRecord = qe
                        //             .fixed_manager()
                        //             .get(id)?
                        //             .ok_or(EngineError::EntityNotFound)?;
                        //         if vert.next_edge_id() == e_id.id {
                        //             vert.set_next_edge_id(edge.from_next_edge_id());
                        //             qe.fixed_manager().update(id, vert)?;
                        //         }
                        //     }
                        //     EdgeDestination::Meta(id) => todo!(),
                        // }
                    }
                }
            }
        }
        Ok(())
    }
}
