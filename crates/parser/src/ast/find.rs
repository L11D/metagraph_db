use crate::{
    EntityTypeDef, FilterDef, FilterHolderDef, Ident, Lexeme, OptionToString, Parsable, ParseError, Parser
};

//FIND (x:VERT in student, person)
//WHERE x.name = "Alice"
#[derive(Debug, Clone)]
pub struct FindStmt {
    pub elements: Vec<PathElement>,
    pub filter: Option<FilterHolderDef>,
}
impl Parsable for FindStmt {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        log::info!("Parsing FIND");
        parser.expect(Lexeme::Find)?;

        let mut elements = Vec::with_capacity(1);

        let first_element = NodePathElement::parse(parser)?;
        elements.push(PathElement::Node(first_element));
        log::info!("Parsed first path element");

        let patterns = EdgeNodePattern::parse_any_count(parser, None);
        for pattern in patterns {
            elements.extend(pattern.elements);
        }

        let filter = FilterHolderDef::parse_probably(parser);

        Ok(Self {
            elements: elements,
            filter: filter,
        })
    }
}

#[cfg(test)]
mod find_stmt_test {
    use crate::ast::parse_test;

    use super::*;

    #[test]
    fn test_parse_find_stmt() {
        let input = r#"
            FIND (x:VERT in student, person), (y:META in person)
        "#;
        let result = parse_test::<FindStmt>(input).unwrap();
    }

    #[test]
    fn test_parse_find_stmt2() {
        let input = r#"
            FIND (x:VERT in student, person), (y:META in person)
            WHERE x.name = "Alice"
        "#;
        let result = parse_test::<FindStmt>(input).unwrap();
    }

    #[test]
    fn test_parse_find_stmt3() {
        let input = r#"
            DELETE (x:VERT in student, person), (y:META in person)
            WHERE x.name = "Alice"
        "#;
        let result = parse_test::<FindStmt>(input).unwrap();
    }
}

#[derive(Debug, Clone)]
struct EdgeNodePattern {
    elements: Vec<PathElement>
}
impl Parsable for EdgeNodePattern {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let edge = EdgePathElement::parse(parser)?;
        let node = NodePathElement::parse(parser)?;
        Ok(Self {
            elements: vec![PathElement::Edge(edge), PathElement::Node(node)]
        })
    }
}   

#[derive(Debug, Clone)]
pub enum PathElement {
    Node(NodePathElement),
    Edge(EdgePathElement)
}

#[derive(Debug, Clone)]
pub struct NodePathElement {
    pub alias: String,
    pub entity_type: Option<EntityTypeDef>,
    pub parents: Vec<String>,
}
impl Parsable for NodePathElement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(Lexeme::LParen)?;
        let alias = Ident::parse(parser)?.to_string();

        let entity_type = if parser.expect_probably(Lexeme::Colon).is_some() {
            let ty = EntityTypeDef::parse(parser)?;
            if ty == EntityTypeDef::Edge {
                Err(ParseError::CantFindEdge)?;
            }
            Some(ty)
        } else {
            None
        };

        let parents = super::parse_parents(parser)?;

        parser.expect(Lexeme::RParen)?;
        Ok(Self {
            alias: alias,
            entity_type: entity_type,
            parents: parents,
        })
    }
}

#[derive(Debug, Clone)]
pub struct EdgePathElement {
    pub alias: String,
}
impl Parsable for EdgePathElement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(Lexeme::Dash)?;
        parser.expect(Lexeme::LBracket)?;
        let alias = Ident::parse(parser)?.to_string();
        parser.expect(Lexeme::RBracket)?;
        parser.expect(Lexeme::Dash)?;
        Ok(Self { alias: alias })
    }
}

// ---------------------------------------------------------------------------
// FindPart
// ---------------------------------------------------------------------------

//(x:VERT in student, person)
#[derive(Debug, Clone)]
pub struct FindPart {
    pub alias: Option<String>,
    pub entity_type: Option<EntityTypeDef>,
    pub parents: Vec<String>,
}
impl Parsable for FindPart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(Lexeme::LParen)?;
        let alias = Ident::parse_probably(parser).to_string();

        let entity_type = if parser.expect_probably(Lexeme::Colon).is_some() {
            let ty = EntityTypeDef::parse(parser)?;
            if ty == EntityTypeDef::Edge {
                Err(ParseError::CantFindEdge)?;
            }
            Some(ty)
        } else {
            None
        };

        let parents = super::parse_parents(parser)?;

        parser.expect(Lexeme::RParen)?;
        Ok(Self {
            alias: alias,
            entity_type: entity_type,
            parents: parents,
        })
    }
}

#[cfg(test)]
mod find_part_test {
    use crate::ast::parse_test;

    use super::*;

    #[test]
    fn test_parse_find_part() {
        let input = r#"(x:VERT in student, person)"#;
        let result = parse_test::<FindPart>(input).unwrap();
        assert_eq!(result.alias, Some("x".into()));
        assert_eq!(result.entity_type, Some(EntityTypeDef::Vertex));
        assert_eq!(result.parents, vec!["student", "person"]);
    }

    #[test]
    fn test_parse_find_part_none_alias() {
        let input = r#"(:VERT in student, person)"#;
        let result = parse_test::<FindPart>(input).unwrap();
        assert_eq!(result.alias, None);
        assert_eq!(result.entity_type, Some(EntityTypeDef::Vertex));
        assert_eq!(result.parents, vec!["student", "person"]);
    }

    #[test]
    fn test_parse_find_part_none_parents() {
        let input = r#"(:VERT)"#;
        let result = parse_test::<FindPart>(input).unwrap();
        assert_eq!(result.alias, None);
        assert_eq!(result.entity_type, Some(EntityTypeDef::Vertex));
        assert!(result.parents.is_empty());
    }

    #[test]
    fn test_parse_find_part_err() {
        let input = r#"(:)"#;
        let result = parse_test::<FindPart>(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_find_part_err2() {
        let input = r#"(x:VERT in )"#;
        let result = parse_test::<FindPart>(input);
        assert!(result.is_err());
    }
}

