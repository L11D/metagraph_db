use std::{
    collections::{BTreeSet, HashMap, HashSet},
    result,
};

use parser::{
    EdgePathElement, EntityTypeDef, FilterDef, FindStmt, NodePathElement, PathElement, PredicateDef,
};

use crate::{
    query::{
        Filter, QueryEngine,
        common::{FullEntityId, QueryContext},
        engine::Executable,
        error::EngineError,
    },
    storage::{
        AttributesRecord, EdgeDestination, EdgeRecord, MetaInfoRecord, MetaRecord, SlottedRecordId,
        VertRecord,
    },
};

impl Executable for FindStmt {
    fn execute(&self, qe: &QueryEngine, ctx: &mut QueryContext) -> Result<(), EngineError> {
        let node_path_elements: Vec<NodePathElement> = self
            .elements
            .iter()
            .filter(|e| matches!(e, PathElement::Node(_)))
            .map(|e| match e {
                PathElement::Node(n) => n.clone(),
                PathElement::Edge(_) => unreachable!(),
            })
            .collect();

        // dbg!(&node_path_elements);

        let edge_path_elements: Vec<EdgePathElement> = self
            .elements
            .iter()
            .filter(|e| matches!(e, PathElement::Edge(_)))
            .map(|e| match e {
                PathElement::Node(_) => unreachable!(),
                PathElement::Edge(e) => e.clone(),
            })
            .collect();

        let filters = if let Some(filer) = &self.filter {
            if filer.is_only_and {
                let mut filters: HashMap<String, Filter> = HashMap::new();
                simplify_filter(&filer.filter, &mut filters)?;
                filters
            } else {
                Err(EngineError::Unexpected(
                    "not implement for not only and filter".into(),
                ))?
            }
        } else {
            HashMap::<String, Filter>::new()
        };

        let first_node = &node_path_elements[0];
        let initial_ids = find_node_ids_with_filters(first_node, &filters, qe)?;
        let mut key = BTreeSet::from_iter(vec![first_node.alias.clone()]);

        for entity_id in initial_ids {
            let mut row = HashMap::new();
            row.insert(first_node.alias.clone(), entity_id);
            let key = BTreeSet::from_iter(vec![first_node.alias.clone()]);
            ctx.add_row(&key, row);
        }

        for (i, edge_path_element) in edge_path_elements.iter().enumerate() {
            let r_path_element = &node_path_elements[i + 1];
            let mut new_rows = Vec::new();

            for row in ctx.get_rows(&key) {
                let l_alias = &node_path_elements[i].alias;
                let l_entity = row.get(l_alias).unwrap();

                let neighbors = get_connected_nodes(l_entity, qe)?;

                for (edge, neighbor) in neighbors {
                    // Проверяем фильтры и тип для "правого" узла
                    if matches_node_criteria(&neighbor, r_path_element, &filters, qe)? {
                        let mut extended_row = row.clone();
                        extended_row.insert(edge_path_element.alias.clone(), edge);
                        extended_row.insert(r_path_element.alias.clone(), neighbor);
                        new_rows.push(extended_row);
                    }
                }
            }

            ctx.delete_rows(&key);
            key.insert(r_path_element.alias.clone());
            key.insert(edge_path_element.alias.clone());

            ctx.set_rows(&key, new_rows);
        }

        Ok(())
    }
}

fn matches_node_criteria(
    neighbor: &FullEntityId,
    node_path_element: &NodePathElement,
    filters: &HashMap<String, Filter>,
    qe: &QueryEngine,
) -> Result<bool, EngineError> {
    if let Some(expected_type) = node_path_element.entity_type {
        if neighbor.ty != expected_type {
            return Ok(false);
        }
    }

    if !node_has_valid_parents(neighbor, node_path_element, qe)? {
        return Ok(false);
    }

    let match_filter = match neighbor.ty {
        EntityTypeDef::Vertex => {
            let vert: VertRecord = qe
                .fixed_manager()
                .get(neighbor.id)?
                .ok_or(EngineError::EntityNotFound)?;
            is_match_filter(
                *neighbor,
                &node_path_element.alias,
                vert.attr_record_id(),
                filters,
                qe,
            )?
        }
        EntityTypeDef::Metavertex => {
            let meta: MetaRecord = qe
                .fixed_manager()
                .get(neighbor.id)?
                .ok_or(EngineError::EntityNotFound)?;
            is_match_filter(
                *neighbor,
                &node_path_element.alias,
                meta.attr_record_id(),
                filters,
                qe,
            )?
        }
        _ => false,
    };

    Ok(match_filter)
}

fn node_has_valid_parents(
    neighbor: &FullEntityId,
    node_path_element: &NodePathElement,
    qe: &QueryEngine,
) -> Result<bool, EngineError> {
    let mut parent_ids = Vec::new();
    for p in node_path_element.parents.iter() {
        parent_ids.push(find_parent_id(qe, p)?);
    }

    for parent_id in parent_ids {
        let parent_mv: MetaRecord = qe
            .fixed_manager()
            .get(parent_id)?
            .ok_or(EngineError::UnknownParent("err".into()))?;

        if !parent_mv.meta_info_id().is_empty() {
            let meta_info: MetaInfoRecord = qe
                .slotted_manager()
                .get(parent_mv.meta_info_id())?
                .ok_or(EngineError::UnknownParent("err".into()))?;

            let is_valid_parent = match neighbor.ty {
                EntityTypeDef::Vertex => meta_info.v_children.contains(&neighbor.id),
                EntityTypeDef::Metavertex => meta_info.mv_children.contains(&neighbor.id),
                _ => false,
            };

            if !is_valid_parent {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    }

    Ok(true)
}

fn get_connected_nodes(
    id: &FullEntityId,
    qe: &QueryEngine,
) -> Result<Vec<(FullEntityId, FullEntityId)>, EngineError> {
    let mut result = Vec::new();

    let mut edge_id = match id.ty {
        EntityTypeDef::Vertex => {
            let vert: VertRecord = qe
                .fixed_manager()
                .get(id.id)?
                .ok_or(EngineError::EntityNotFound)?;
            vert.next_edge_id()
        }
        EntityTypeDef::Metavertex => {
            let meta: MetaRecord = qe
                .fixed_manager()
                .get(id.id)?
                .ok_or(EngineError::EntityNotFound)?;
            meta.next_edge_id()
        }
        _ => panic!("Unexpected entity type"),
    };

    if edge_id.is_empty() {
        return Ok(result);
    }

    loop {
        let edge_record: EdgeRecord = qe
            .fixed_manager()
            .get(edge_id)?
            .ok_or(EngineError::EntityNotFound)?;

        let next_edge_id = if edge_record.from().id() == id.id {
            match edge_record.to() {
                EdgeDestination::Vert(id) => {
                    result.push((FullEntityId::new(edge_id, EntityTypeDef::Edge), FullEntityId::new(id, EntityTypeDef::Vertex)));
                }
                EdgeDestination::Meta(id) => {
                    result.push((FullEntityId::new(edge_id, EntityTypeDef::Edge), FullEntityId::new(id, EntityTypeDef::Metavertex)));
                }
            }
            edge_record.from_next_edge_id()
        } else {
            edge_record.to_next_edge_id()
        };

        if next_edge_id.is_empty() {
            break;
        }

        edge_id = next_edge_id;
    }

    Ok(result)
}

fn find_node_ids_with_filters(
    node: &NodePathElement,
    filters: &HashMap<String, Filter>,
    qe: &QueryEngine,
) -> Result<Vec<FullEntityId>, EngineError> {
    let mut result = Vec::new();

    if node.parents.is_empty() {
        if matches!(node.entity_type, Some(EntityTypeDef::Vertex)) || node.entity_type.is_none() {
            for (vert, id) in qe.fixed_manager().iter::<VertRecord>() {
                let f_id = FullEntityId::new(id, EntityTypeDef::Vertex);
                if is_match_filter(f_id, &node.alias, vert.attr_record_id(), filters, qe)? {
                    result.push(f_id);
                }
            }
        }

        if matches!(node.entity_type, Some(EntityTypeDef::Metavertex)) || node.entity_type.is_none()
        {
            for (meta, id) in qe.fixed_manager().iter::<MetaRecord>() {
                let f_id = FullEntityId::new(id, EntityTypeDef::Metavertex);
                if is_match_filter(f_id, &node.alias, meta.attr_record_id(), filters, qe)? {
                    result.push(f_id);
                }
            }
        }
    } else {
        let (vert_ids, meta_ids) = get_child_ids(qe, &node)?;
        for vert_id in vert_ids {
            let vert: VertRecord = qe
                .fixed_manager()
                .get(vert_id)?
                .ok_or(EngineError::EntityNotFound)?;

            let f_id = FullEntityId::new(vert_id, EntityTypeDef::Vertex);
            if is_match_filter(f_id, &node.alias, vert.attr_record_id(), filters, qe)? {
                result.push(f_id);
            }
        }

        for meta_id in meta_ids {
            let meta: MetaRecord = qe
                .fixed_manager()
                .get(meta_id)?
                .ok_or(EngineError::EntityNotFound)?;

            let f_id = FullEntityId::new(meta_id, EntityTypeDef::Metavertex);
            if is_match_filter(f_id, &node.alias, meta.attr_record_id(), filters, qe)? {
                result.push(f_id);
            }
        }
    }

    Ok(result)
}

fn is_match_filter(
    f_id: FullEntityId,
    alias: &str,
    attr_id: SlottedRecordId,
    filters: &HashMap<String, Filter>,
    qe: &QueryEngine,
) -> Result<bool, EngineError> {
    if let Some(filter) = filters.get(alias) {
        if !attr_id.is_empty() {
            let attrs: AttributesRecord = qe
                .slotted_manager()
                .get(attr_id)?
                .ok_or(EngineError::EntityNotFound)?;

            return Ok(filter.matches(f_id, &attrs.attrs)?);
        }
        return Ok(false);
    } else {
        return Ok(true);
    }
}

fn get_child_ids(
    qe: &QueryEngine,
    node: &NodePathElement,
) -> Result<(Vec<SlottedRecordId>, Vec<SlottedRecordId>), EngineError> {
    let mut parent_ids = Vec::new();
    for p in node.parents.iter() {
        parent_ids.push(find_parent_id(qe, p)?);
    }
    let mut v_results = Vec::new();
    let mut mv_results = Vec::new();
    for p in parent_ids {
        let parent_mv: MetaRecord = qe
            .fixed_manager()
            .get(p)?
            .ok_or(EngineError::UnknownParent("err".into()))?;
        if !parent_mv.meta_info_id().is_empty() {
            let meta_info: MetaInfoRecord = qe
                .slotted_manager()
                .get(parent_mv.meta_info_id())?
                .ok_or(EngineError::UnknownParent("err".into()))?;
            match node.entity_type {
                Some(EntityTypeDef::Vertex) => v_results.extend(meta_info.v_children),
                Some(EntityTypeDef::Metavertex) => mv_results.extend(meta_info.mv_children),
                Some(EntityTypeDef::Edge) => {
                    Err(EngineError::Unexpected("Unexpected edge".into()))?
                }
                None => {
                    v_results.extend(meta_info.v_children);
                    mv_results.extend(meta_info.mv_children);
                }
            }
        }
    }

    Ok((v_results, mv_results))
}

fn find_parent_id(qe: &QueryEngine, parent_name: &str) -> Result<SlottedRecordId, EngineError> {
    qe.naming_manager()
        .get(parent_name)
        .ok_or(EngineError::UnknownParent(parent_name.to_string()))
}

fn simplify_filter(
    filter: &FilterDef,
    out: &mut HashMap<String, Filter>,
) -> Result<(), EngineError> {
    match filter {
        FilterDef::Leaf(predicate_def) => {
            let current = out.get(&predicate_def.alias);
            let new = compute_filter_for_add_to_map(current, predicate_def)?;
            out.insert(predicate_def.alias.to_string(), new);
            return Ok(());
        }
        FilterDef::And(filter_defs) => {
            for f in filter_defs {
                simplify_filter(f, out)?;
            }
            return Ok(());
        }
        FilterDef::Or(_) => {
            return Err(EngineError::Unexpected("Unexpected or".to_string()));
        }
        FilterDef::Not(_) => {
            return Err(EngineError::Unexpected("Unexpected not".to_string()));
        }
    }

    fn compute_filter_for_add_to_map(
        filter: Option<&Filter>,
        predicate_def: &PredicateDef,
    ) -> Result<Filter, EngineError> {
        if let Some(current) = filter {
            match current {
                Filter::Leaf(_) => {
                    let new = Filter::Leaf(predicate_def.into());
                    return Ok(Filter::And(vec![current.clone(), new]));
                }
                Filter::And(filters) => {
                    let mut old = filters.clone();
                    old.push(Filter::Leaf(predicate_def.into()));
                    return Ok(Filter::And(old));
                }
                Filter::Or(_) => {
                    return Err(EngineError::Unexpected("Unexpected or".to_string()));
                }
                Filter::Not(_) => {
                    return Err(EngineError::Unexpected("Unexpected not".to_string()));
                }
            }
        } else {
            return Ok(Filter::Leaf(predicate_def.into()));
        }
    }
}
