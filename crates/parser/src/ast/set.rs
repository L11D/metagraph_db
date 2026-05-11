use crate::{ParseError, ValueDef, lexer::Lexeme, parser::{Ident, Parsable, Parser}};

// SET {
//    x.name = "Bob",
//}
#[derive(Debug, Clone)]
pub struct SetStmt {
    pub parts: Vec<SetPart>,
}
impl Parsable for SetStmt {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(Lexeme::Set)?;
        parser.expect(Lexeme::LBrace)?;
        let parts = SetPart::parse_one_or_more(parser, Some(Lexeme::Comma))?;
        parser.expect(Lexeme::RBrace)?;
        Ok(Self { parts })
    }
}

// x.name = "Bob",
#[derive(Debug, Clone)]
pub struct SetPart {
    pub alias: String,
    pub attribute_name: String,
    pub value: ValueDef,
}
impl Parsable for SetPart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let alias: String = Ident::parse(parser)?.to_string();
        parser.expect(Lexeme::Dot)?;
        let attribute_name = Ident::parse(parser)?.to_string();
        parser.expect(Lexeme::Equal)?;
        let value = ValueDef::parse(parser)?;
        Ok(Self {
            alias,
            attribute_name,
            value
        })
    }
}