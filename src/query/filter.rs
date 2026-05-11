use std::collections::HashMap;

use parser::{ComparisonDef, PredicateDef, ValueDef};

use crate::{
    attribute::{Attribute, AttributeError, CompOp, Value}, node_id::NodeId, query::common::FullEntityId, storage::AttributesRecord
};

pub trait Attributed {
    fn get_attribute(&self, name: &str) -> Option<&Attribute>;
}


#[derive(Debug, Clone)]
pub enum Filter {
    Leaf(Predicate),
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>),
}
impl Filter {
    pub fn matches(&self, id: FullEntityId, attrs: &HashMap<String, Attribute>) -> Result<bool, AttributeError> {
        match self {
            Filter::Leaf(pred) => {
                let attribute = attrs.get(&pred.attribute_name);
                pred.evaluate(id, attribute)
            }
            Filter::And(filters) => {
                // Все условия должны быть true
                filters.iter().try_fold(
                    true,
                    |acc, f| {
                        if acc { f.matches(id, attrs) } else { Ok(false) }
                    },
                )
            }
            Filter::Or(filters) => {
                // Хотя бы одно условие должно быть true
                filters.iter().try_fold(
                    false,
                    |acc, f| {
                        if !acc { f.matches(id, attrs) } else { Ok(true) }
                    },
                )
            }
            Filter::Not(filter) => Ok(!filter.matches(id, attrs)?),
        }
    }

    

}

#[derive(Debug, Clone)]
pub struct Predicate {
    pub attribute_name: String,
    pub comparison: Option<Comparison>,
}
impl Predicate {
    fn evaluate(&self, id: FullEntityId, attr: Option<&Attribute>) -> Result<bool, AttributeError> {
        match (attr, &self.comparison) {
            (Some(a), Some(c)) => a.compare_value(&c.value, c.op),
            (Some(a), None) => Ok(true),
            (None, Some(c)) => {
                if self.attribute_name == "id".to_string() {
                    let value_id = Value::Text(id.to_string());
                    value_id.compare(&c.value, c.op)
                } else {
                    Ok(false)
                }
            }
            (None, None) => Ok(false),
        }
    }
}

impl From<&PredicateDef> for Predicate {
    fn from(value: &PredicateDef) -> Self {
        let comp = if let Some(c) = &value.comparison {
            Some(c.into())
        } else {None};

        Self { 
            attribute_name: value.attribute_name.to_string(), 
            comparison: comp, 
        }
    }
}

#[derive(Debug, Clone)]
pub struct Comparison {
    pub value: Value,
    pub op: CompOp,
}
impl From<&ComparisonDef> for Comparison {
    fn from(value: &ComparisonDef) -> Self {
        Self { value: value.value.clone().into(), op: value.op.clone().into() }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::collections::HashMap;

//     // Моковая реализация для теста
//     struct MockItem {
//         attrs: HashMap<String, Attribute>,
//     }
//     impl Attributed for MockItem {
//         fn get_attribute(&self, name: &str) -> Option<&Attribute> {
//             self.attrs.get(name)
//         }
//     }

//     #[test]
//     fn test_filter_logic() {
//         let mut attrs: HashMap<String, Attribute> = HashMap::new();
//         attrs.insert("a".to_string(), "gg".into());
//         attrs.insert("b".to_string(), "ff".into());

//         let item = MockItem { attrs: attrs }; // Пустой атрибут

//         // Тест Not
//         let not_filter = Filter::Not(Box::new(Filter::Leaf(Predicate {
//             attribute_name: "nonexistent".into(),
//             comparison: None,
//         })));
//         // Leaf вернет false (атрибута нет), Not инвертирует в true
//         assert_eq!(not_filter.matches(&"1".into(), &item).unwrap(), true);

//         let and_filter = Filter::And(vec![
//             Filter::Leaf(Predicate {
//                 attribute_name: "id".into(),
//                 comparison: Some(Comparison {
//                     value: "1".into(),
//                     op: CompOp::Equal,
//                 }),
//             }),
//             Filter::Leaf(Predicate {
//                 attribute_name: "b".into(),
//                 comparison: Some(Comparison {
//                     value: "ff".into(),
//                     op: CompOp::Equal,
//                 }),
//             }),
//         ]);
//         assert_eq!(and_filter.matches(&"1".into(), &item).unwrap(), true);
//     }
// }

pub struct Chain {
    
}

// #[derive(Debug, Clone)]
// pub enum FilterChain {
//     Leaf(Predicate),
//     And(Vec<Filter>),
//     Or(Vec<Filter>),
//     Not(Box<Filter>),
// }
// impl FilterChain {
//     pub fn matches<T: Attributed>(&self, id: &NodeId, item: &T) -> Result<bool, AttributeError> {
//         match self {
//             FilterChain::Leaf(pred) => {
//                 let attribute = item.get_attribute(&pred.attribute_name);
//                 pred.evaluate(id, attribute)
//             }
//             FilterChain::And(filters) => {
//                 // Все условия должны быть true
//                 filters.iter().try_fold(
//                     true,
//                     |acc, f| {
//                         if acc { f.matches(id, item) } else { Ok(false) }
//                     },
//                 )
//             }
//             FilterChain::Or(filters) => {
//                 // Хотя бы одно условие должно быть true
//                 filters.iter().try_fold(
//                     false,
//                     |acc, f| {
//                         if !acc { f.matches(id, item) } else { Ok(true) }
//                     },
//                 )
//             }
//             FilterChain::Not(filter) => Ok(!filter.matches(id, item)?),
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct PredicateChain {
//     pub attribute_name: String,
//     pub comparison: Option<Comparison>,
// }
// impl PredicateChain {
//     fn evaluate(&self, id: &NodeId, attr: Option<&Attribute>) -> Result<bool, AttributeError> {
//         match (attr, &self.comparison) {
//             (Some(a), Some(c)) => a.compare_value(&c.value, c.op),
//             (Some(a), None) => Ok(true),
//             (None, Some(c)) => {
//                 if self.attribute_name == "id".to_string() {
//                     let value_id: Value = id.to_string().into();
//                     value_id.compare(&c.value, c.op)
//                 } else {
//                     Ok(false)
//                 }
//             }
//             (None, None) => Ok(false),
//         }
//     }
// }