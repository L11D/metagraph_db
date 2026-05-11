use crate::{ParseError, lexer::Lexeme, parser::{Ident, Parsable, Parser}};

// RETURN a.name, b.name
#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub parts: Vec<ReturnPart>,
}
impl Parsable for ReturnStmt {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(Lexeme::Return)?;
        let parts = ReturnPart::parse_one_or_more(parser, Some(Lexeme::Comma))?;
        Ok(Self { parts })
    }
}

// a.name
#[derive(Debug, Clone)]
pub struct ReturnPart {
    pub alias: String,
    pub attribute_name: Option<String>,
}

impl Parsable for ReturnPart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let alias: String = Ident::parse(parser)?.to_string();

        let attr_name = if parser.expect_probably(Lexeme::Dot).is_some() {
            Some(Ident::parse(parser)?.to_string())
        } else {
            None
        };

        Ok(Self {
            alias: alias,
            attribute_name: attr_name
        })
    }
}