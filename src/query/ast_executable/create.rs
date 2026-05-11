use std::collections::{BTreeSet, HashMap, HashSet};

use parser::{
    AttributeDef, ConstraintDef, CreateEdge, CreateMetavertex, CreateStmt, CreateVertex,
    EntityTypeDef,
};

use crate::{
    attribute::{Attribute, AttributeType},
    metagraph::ChildConstraint,
    query::{
        FullEntityId, QueryEngine, common::QueryContext, engine::Executable, error::EngineError,
    },
    storage::{
        AttributesRecord, EdgeDestination, EdgeRecord, MetaInfoRecord, MetaRecord, ParentsRecord,
        SlottedRecordId, VertRecord,
    },
};

impl Executable for CreateStmt {
    fn execute(&self, qe: &QueryEngine, ctx: &mut QueryContext) -> Result<(), EngineError> {
        match self {
            CreateStmt::Vertex(create_vertex) => create_vertex.execute(qe, ctx),
            CreateStmt::Metavertex(create_metavertex) => create_metavertex.execute(qe, ctx),
            CreateStmt::Edge(create_edge) => create_edge.execute(qe, ctx),
        }
    }
}

impl Executable for CreateVertex {
    fn execute(&self, qe: &QueryEngine, ctx: &mut QueryContext) -> Result<(), EngineError> {
        let parents_ids = get_parents_ids(&self.parents, qe)?;
        let attrs = convert_attrs(&self.attributes);

        check_parents_constraints(&parents_ids, &attrs, &HashSet::new(), qe)?;

        let parents = create_parents(parents_ids.clone(), qe)?;
        let attributes = create_attrs(attrs, qe)?;

        let vert_record = VertRecord::new(attributes, SlottedRecordId::Empty, parents);
        let id = qe.fixed_manager().insert(vert_record)?;
        update_parents(&id, &parents_ids, EntityTypeDef::Vertex, qe)?;

        ctx.add_entity(&self.alias, FullEntityId::new(id, EntityTypeDef::Vertex));
        Ok(())
    }
}

impl Executable for CreateMetavertex {
    fn execute(&self, qe: &QueryEngine, ctx: &mut QueryContext) -> Result<(), EngineError> {
        if let Some(_) = qe.naming_manager().get(&self.name) {
            return Err(EngineError::MetaAlreadyExists(self.name.clone()));
        }

        let parents_ids = get_parents_ids(&self.parents, qe)?;
        let attrs = convert_attrs(&self.attributes);
        let constraints = convert_constraints(&self.constraints);

        check_parents_constraints(&parents_ids, &attrs, &constraints, qe)?;

        let parents = create_parents(parents_ids.clone(), qe)?;
        let attributes = create_attrs(attrs, qe)?;

        let meta_info = MetaInfoRecord::new(HashSet::new(), HashSet::new(), constraints);
        let meta_info_id = qe.slotted_manager().insert(meta_info)?;

        let meta_record =
            MetaRecord::new(attributes, SlottedRecordId::Empty, parents, meta_info_id);
        let id = qe.fixed_manager().insert(meta_record)?;
        qe.naming_manager().insert(self.name.clone(), id);
        update_parents(&id, &parents_ids, EntityTypeDef::Metavertex, qe)?;


        ctx.add_entity(
            &self.alias,
            FullEntityId::new(id, EntityTypeDef::Metavertex),
        );

        Ok(())
    }
}

fn update_parents(
    id: &SlottedRecordId,
    parents_ids: &Vec<SlottedRecordId>,
    ty: EntityTypeDef,
    qe: &QueryEngine,
) -> Result<(), EngineError> {
    for parent_id in parents_ids {
        let mut parent: MetaRecord = qe.fixed_manager().get(*parent_id)?.unwrap();
        if !parent.meta_info_id().is_empty() {
            let mut parent_info: MetaInfoRecord =
                qe.slotted_manager().get(parent.meta_info_id())?.unwrap();

            match ty {
                EntityTypeDef::Vertex => {
                    parent_info.v_children.insert(*id);
                }
                EntityTypeDef::Metavertex => {
                    parent_info.mv_children.insert(*id);
                }
                EntityTypeDef::Edge => todo!(),
            }

            let new_meta_info_id = qe
                .slotted_manager()
                .update(parent.meta_info_id(), parent_info)?;
            if new_meta_info_id != parent.meta_info_id() {
                parent.set_meta_info_id(new_meta_info_id);
                qe.fixed_manager().update(*parent_id, parent)?;
            }
        } else {
            let mut parent_info =
                MetaInfoRecord::new(HashSet::new(), HashSet::new(), HashSet::new());
            match ty {
                EntityTypeDef::Vertex => {
                    parent_info.v_children.insert(*id);
                }
                EntityTypeDef::Metavertex => {
                    parent_info.mv_children.insert(*id);
                }
                EntityTypeDef::Edge => todo!(),
            }
            let meta_info_id = qe.slotted_manager().insert(parent_info)?;
            parent.set_meta_info_id(meta_info_id);
            qe.fixed_manager().update(*parent_id, parent)?;
        }
    }

    Ok(())
}

fn create_parents(
    parents_ids: Vec<SlottedRecordId>,
    qe: &QueryEngine,
) -> Result<SlottedRecordId, EngineError> {
    if parents_ids.is_empty() {
        Ok(SlottedRecordId::Empty)
    } else if parents_ids.len() == 1 {
        Ok(parents_ids[0])
    } else {
        let parents_record = ParentsRecord::new(HashSet::from_iter(parents_ids.into_iter()));
        Ok(qe.slotted_manager().insert(parents_record)?)
    }
}

fn check_parents_constraints(
    parents_ids: &Vec<SlottedRecordId>,
    attributes: &HashMap<String, Attribute>,
    child_constraints: &HashSet<ChildConstraint>,
    qe: &QueryEngine,
) -> Result<(), EngineError> {
    for parent_id in parents_ids {
        let parent: MetaRecord = qe.fixed_manager().get(*parent_id)?.unwrap();
        if !parent.meta_info_id().is_empty() {
            let parent_info: MetaInfoRecord =
                qe.slotted_manager().get(parent.meta_info_id())?.unwrap();
            for constr in parent_info.constraints {
                if !child_constraints.contains(&constr) {
                    constr.check(attributes)?;
                }
            }
        }
    }

    Ok(())
}

fn create_attrs(
    attributes: HashMap<String, Attribute>,
    qe: &QueryEngine,
) -> Result<SlottedRecordId, EngineError> {
    if attributes.is_empty() {
        Ok(SlottedRecordId::Empty)
    } else {
        let result = qe
            .slotted_manager()
            .insert(AttributesRecord::new(attributes))?;
        Ok(result)
    }
}

fn convert_attrs(attributes: &Vec<AttributeDef>) -> HashMap<String, Attribute> {
    attributes
        .iter()
        .map(|a| {
            let name = a.name.clone();
            let attr: Attribute = a.into();
            (name, attr)
        })
        .collect()
}

fn get_parents_ids(
    parents: &Vec<String>,
    qe: &QueryEngine,
) -> Result<Vec<SlottedRecordId>, EngineError> {
    parents
        .iter()
        .map(|p| {
            qe.naming_manager()
                .get(p)
                .ok_or_else(|| EngineError::UnknownParent(p.clone()))
        })
        .collect::<Result<Vec<_>, _>>()
}

fn convert_constraints(constraints: &Vec<ConstraintDef>) -> HashSet<ChildConstraint> {
    constraints
        .iter()
        .map(|c| ChildConstraint::HasAttribute {
            name: c.name.clone(),
            atype: AttributeType::new(c.primitive_type.into(), c.nullable),
        })
        .collect()
}

impl Executable for CreateEdge {
    fn execute(&self, qe: &QueryEngine, ctx: &mut QueryContext) -> Result<(), EngineError> {
        let from_nodes_ids = ctx.get_by_alias(&self.from);
        if from_nodes_ids.is_empty() {
            return Err(EngineError::UndefinedAlias(self.from.clone()));
        }

        let to_nodes_ids = ctx.get_by_alias(&self.to);
        if to_nodes_ids.is_empty() {
            return Err(EngineError::UndefinedAlias(self.to.clone()));
        }

        let attrs = convert_attrs(&self.attributes);
        let attributes = create_attrs(attrs, qe)?;

        let mut rows: Vec<HashMap<String, FullEntityId>> = Vec::with_capacity(from_nodes_ids.len() * to_nodes_ids.len());

        for f in from_nodes_ids {
            for t in &to_nodes_ids {

                let mut row = HashMap::new();

                let edge_record = EdgeRecord::new_last(
                    EdgeDestination::Vert(f.id.clone()),
                    EdgeDestination::Vert(t.id.clone()),
                    attributes,
                );
                let edge_id = qe.fixed_manager().insert(edge_record)?;
                row.insert(self.alias.clone(), FullEntityId::new(edge_id, EntityTypeDef::Edge));

                match f.ty {
                    EntityTypeDef::Vertex => {
                        let mut from_vert: VertRecord = qe.fixed_manager().get(*&f.id)?.unwrap();
                        if from_vert.next_edge_id().is_empty() {
                            from_vert.set_next_edge_id(edge_id.clone());
                            qe.fixed_manager().update(*&f.id, from_vert)?;
                        } else {
                            let (from_last_edge_id, mut from_last_edge) =
                                get_last_edge(*&f.id, from_vert.next_edge_id(), qe)?;
                            from_last_edge.set_from_next_edge_id(edge_id.clone());
                            qe.fixed_manager()
                                .update(from_last_edge_id, from_last_edge)?;
                        }
                        row.insert(self.from.clone(), FullEntityId::new(*&f.id, EntityTypeDef::Vertex));

                    }
                    EntityTypeDef::Metavertex => todo!(),
                    EntityTypeDef::Edge => todo!(),
                }

                match t.ty {
                    EntityTypeDef::Vertex => {
                        let mut to_vert: VertRecord = qe.fixed_manager().get(*&t.id)?.unwrap();
                        if to_vert.next_edge_id().is_empty() {
                            to_vert.set_next_edge_id(edge_id.clone());
                            qe.fixed_manager().update(*&t.id, to_vert)?;
                        } else {
                            let (to_last_edge_id, mut to_last_edge) =
                                get_last_edge(*&t.id, to_vert.next_edge_id(), qe)?;
                            to_last_edge.set_from_next_edge_id(edge_id.clone());
                            qe.fixed_manager().update(to_last_edge_id, to_last_edge)?;
                        }
                        row.insert(self.to.clone(), FullEntityId::new(*&t.id, EntityTypeDef::Vertex));
                    }
                    EntityTypeDef::Metavertex => todo!(),
                    EntityTypeDef::Edge => todo!(),
                }

                rows.push(row);
            }
        }

        let key = BTreeSet::from_iter(vec![self.alias.clone(), self.from.clone(), self.to.clone()]);
        ctx.set_rows(&key, rows);

        Ok(())
    }
}

fn get_last_edge(
    node_id: SlottedRecordId,
    first_edge_id: SlottedRecordId,
    qe: &QueryEngine,
) -> Result<(SlottedRecordId, EdgeRecord), EngineError> {
    let mut edge_record: EdgeRecord = qe.fixed_manager().get(first_edge_id)?.unwrap();
    let mut edge_id = first_edge_id;

    loop {
        let next_edge_id = if edge_record.from().id() == node_id {
            edge_record.from_next_edge_id()
        } else if edge_record.to().id() == node_id {
            edge_record.to_next_edge_id()
        } else {
            panic!("Node is not part of this edge");
        };

        if next_edge_id.is_empty() {
            break;
        } else {
            edge_id = next_edge_id;
            edge_record = qe.fixed_manager().get(edge_id)?.unwrap();
        }
    }

    Ok((edge_id, edge_record))
}
