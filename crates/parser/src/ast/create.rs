use crate::{AttributeDef, ConstraintDef, Ident, Lexeme, OptionToString, Parsable, ParseError, Parser};

#[derive(Debug, Clone)]
pub enum CreateStmt {
    Vertex(CreateVertex),
    Metavertex(CreateMetavertex),
    Edge(CreateEdge),
}
impl Parsable for CreateStmt {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(Lexeme::Create)?;

        if let Some(v) = CreateVertex::parse_probably(parser) {
            Ok(Self::Vertex(v))
        } else if let Some(mv) = CreateMetavertex::parse_probably(parser) {
            Ok(Self::Metavertex(mv))
        } else if let Some(e) = CreateEdge::parse_probably(parser) {
            Ok(Self::Edge(e))
        } else {
            Err(ParseError::InvalidCreate)
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateMetavertex {
    pub name: String,
    pub alias: String,
    pub parents: Vec<String>,
    pub attributes: Vec<AttributeDef>,
    pub constraints: Vec<ConstraintDef>,
}
impl Parsable for CreateMetavertex {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let alias: String = Ident::parse(parser)?.to_string();
        parser.expect(Lexeme::Colon)?;
        parser.expect(Lexeme::Meta)?;

        let name: String = Ident::parse(parser)?.to_string();
        let parents = super::parse_parents(parser)?;

        parser.expect(Lexeme::LBrace)?;
        let attributes = AttributeDef::parse_any_count(parser, Some(Lexeme::Comma));
        parser.expect(Lexeme::RBrace)?;

        let constraints = if let Some(_) = parser.expect_probably(Lexeme::Constraint) {
            parser.expect(Lexeme::Attribute)?;
            parser.expect(Lexeme::LBrace)?;
            let constraints = ConstraintDef::parse_any_count(parser, Some(Lexeme::Comma));
            parser.expect(Lexeme::RBrace)?;
            constraints
        } else {
            Vec::new()
        };

        Ok(Self {
            name: name,
            alias: alias,
            parents: parents,
            attributes: attributes,
            constraints: constraints,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CreateVertex {
    pub alias: String,
    pub parents: Vec<String>,
    pub attributes: Vec<AttributeDef>,
}
impl Parsable for CreateVertex {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let alias: String = Ident::parse(parser)?.to_string();
        parser.expect(Lexeme::Colon)?;
        parser.expect(Lexeme::Vert)?;

        let parents = super::parse_parents(parser)?;

        parser.expect(Lexeme::LBrace)?;

        let attributes = AttributeDef::parse_any_count(parser, Some(Lexeme::Comma));

        parser.expect(Lexeme::RBrace)?;

        Ok(Self {
            alias: alias,
            parents: parents,
            attributes: attributes,
        })
    }
}
#[derive(Debug, Clone)]
pub struct CreateEdge {
    pub alias: String,
    pub from: String,
    pub to: String,
    pub attributes: Vec<AttributeDef>,
}
impl Parsable for CreateEdge {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let alias: String = Ident::parse(parser)?.to_string();
        parser.expect(Lexeme::Colon)?;
        parser.expect(Lexeme::Edge)?;

        let from: String = Ident::parse(parser)?.to_string();
        parser.expect(Lexeme::Dash)?;
        parser.expect(Lexeme::Dash)?;
        let to: String = Ident::parse(parser)?.to_string();

        parser.expect(Lexeme::LBrace)?;
        let attributes = AttributeDef::parse_any_count(parser, Some(Lexeme::Comma));
        parser.expect(Lexeme::RBrace)?;

        Ok(Self {
            alias: alias,
            from: from,
            to: to,
            attributes: attributes,
        })
    }
}
