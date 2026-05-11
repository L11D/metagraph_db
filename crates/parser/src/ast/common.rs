use crate::{Ident, Lexeme, Parsable, ParseError, Parser};

#[cfg(test)]
macro_rules! parse_test {
    ($input:expr, $parser:expr) => {{
        let lexemes = crate::interpreter::lexer::parse($input).unwrap();
        let mut parser = crate::interpreter::parser::Parser::new(&lexemes);
        let result = $parser(&mut parser);
        parser.expect(Lexeme::EOF)?;
        result
    }};

    // С проверкой результата
    ($input:expr, $parser:path, assert $expected:expr) => {{
        let result = parse_test!($input, $parser);
        assert_eq!(result?, $expected);
    }};

    // С проверкой на ошибку
    ($input:expr, $parser:path, expect_err) => {{
        let result = parse_test!($input, $parser);
        assert!(result.is_err());
    }};
}

pub fn parse_test<T: Parsable>(input: &str) -> Result<T, ParseError> {
    let lexemes = crate::lexer::parse(input).unwrap();
    let mut parser = crate::Parser::new(&lexemes);
    let result = T::parse(&mut parser)?;
    parser.expect(crate::lexer::Lexeme::EOF)?;
    Ok(result)
}

// ---------------------------------------------------------------------------
// AttributeDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct AttributeDef {
    pub name: String,
    pub primitive_type: PrimitiveTypeDef,
    pub nullable: bool,
    pub value: ValueDef,
}
impl Parsable for AttributeDef {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let name = Ident::parse(parser)?.to_string();
        parser.expect(Lexeme::Colon)?;
        let ptype = PrimitiveTypeDef::parse(parser)?;
        let nullable = parser.expect_probably(Lexeme::Question).is_some();
        parser.expect(Lexeme::Equal)?;
        let value = ValueDef::parse(parser)?;
        Ok(Self {
            name: name,
            primitive_type: ptype,
            nullable: nullable,
            value: value,
        })
    }
}

#[cfg(test)]
mod attribute_def_test {
    use super::*;

    #[test]
    fn test_parse_attribute_def() {
        let input = "age: int32? = 30";
        let result = parse_test::<AttributeDef>(input).unwrap();
        assert_eq!(result.name, "age");
        assert_eq!(result.primitive_type, PrimitiveTypeDef::Int32);
        assert!(result.nullable);
        assert_eq!(result.value, ValueDef::Int32(30));
    }

    #[test]
    fn test_parse_attribute_def_nullable_false() {
        let input = "age: int32 = 30";
        let result = parse_test::<AttributeDef>(input).unwrap();
        assert_eq!(result.name, "age");
        assert_eq!(result.primitive_type, PrimitiveTypeDef::Int32);
        assert_eq!(result.nullable, false);
        assert_eq!(result.value, ValueDef::Int32(30));
    }

    #[test]
    fn test_parse_attribute_def_err() {
        let input = "age int32 = 30";
        let result = parse_test::<AttributeDef>(input);
        assert!(result.is_err());
    }
}

// ---------------------------------------------------------------------------
// ConstraintDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct ConstraintDef {
    pub name: String,
    pub primitive_type: PrimitiveTypeDef,
    pub nullable: bool,
}
impl Parsable for ConstraintDef {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let name = Ident::parse(parser)?.to_string();
        parser.expect(Lexeme::Colon)?;
        let ptype = PrimitiveTypeDef::parse(parser)?;
        let nullable = parser.expect_probably(Lexeme::Question).is_some();
        Ok(Self {
            name: name,
            primitive_type: ptype,
            nullable: nullable,
        })
    }
}

// ---------------------------------------------------------------------------
// ValueDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum ValueDef {
    Int32(i32),
    Text(String),
    Bool(bool),
    Null,
}
impl ValueDef {
    pub fn is_support_comp_op(&self, comp_op: &CompOpDef) -> bool {
        let simpl_comp_ops = vec![CompOpDef::Equal, CompOpDef::NotEqual];
        match self {
            ValueDef::Int32(_) => true,
            ValueDef::Text(_) => simpl_comp_ops.contains(comp_op),
            ValueDef::Bool(_) => simpl_comp_ops.contains(comp_op),
            ValueDef::Null => simpl_comp_ops.contains(comp_op),
        }
    }
}
impl Parsable for ValueDef {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.next() {
            Some(Lexeme::Number(n)) => Ok(Self::Int32(n as i32)),
            Some(Lexeme::String(s)) => Ok(Self::Text(s)),
            Some(Lexeme::Bool(b)) => Ok(Self::Bool(b)),
            Some(Lexeme::Null) => Ok(Self::Null),
            Some(other) => Err(ParseError::UnexpectedNode(
                "Literal".to_string(),
                other.to_string(),
            )),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}

// ---------------------------------------------------------------------------
// CompOpDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum CompOpDef {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}
impl Parsable for CompOpDef {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.next() {
            Some(Lexeme::Equal) => Ok(Self::Equal),
            Some(Lexeme::NotEqual) => Ok(Self::NotEqual),
            Some(Lexeme::Less) => Ok(Self::Less),
            Some(Lexeme::LessOrEqual) => Ok(Self::LessOrEqual),
            Some(Lexeme::Greater) => Ok(Self::Greater),
            Some(Lexeme::GreaterOrEqual) => Ok(Self::GreaterOrEqual),
            Some(other) => Err(ParseError::UnexpectedNode(
                "Comparison operator".to_string(),
                other.to_string(),
            )),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}

// ---------------------------------------------------------------------------
// PrimitiveTypeDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveTypeDef {
    Int32,
    Text,
    Bool,
}
impl Parsable for PrimitiveTypeDef {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.next() {
            Some(Lexeme::TypeInt32) => Ok(Self::Int32),
            Some(Lexeme::TypeText) => Ok(Self::Text),
            Some(Lexeme::TypeBool) => Ok(Self::Bool),
            Some(other) => Err(ParseError::UnknownType(other.to_string())),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}

// ---------------------------------------------------------------------------
// EntityTypeDef
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Copy, Hash, Eq)]
pub enum EntityTypeDef {
    Vertex,
    Metavertex,
    Edge,
}
impl Parsable for EntityTypeDef {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.next() {
            Some(Lexeme::Meta) => Ok(Self::Metavertex),
            Some(Lexeme::Edge) => Ok(Self::Edge),
            Some(Lexeme::Vert) => Ok(Self::Vertex),
            Some(other) => Err(ParseError::UnexpectedNode(
                "Entity Type (META, EDGE, VERT)".to_string(),
                other.to_string(),
            )),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}

pub fn parse_alias_and_attribute_name(
    parser: &mut Parser,
) -> Result<(Option<String>, String), ParseError> {
    let first_ident = Ident::parse(parser)?.to_string();
    if parser.expect_probably(Lexeme::Dot).is_some() {
        let name = Ident::parse(parser)?.to_string();
        return Ok((Some(first_ident), name));
    }
    Ok((None, first_ident))
}

pub fn parse_parents(parser: &mut Parser) -> Result<Vec<String>, ParseError> {
    if parser.expect_probably(Lexeme::In).is_some() {
        let parents = Ident::parse_one_or_more(parser, Some(Lexeme::Comma))?
            .into_iter()
            .map(|i| i.to_string())
            .collect();
        return Ok(parents);
    }
    Ok(Vec::new())
}
