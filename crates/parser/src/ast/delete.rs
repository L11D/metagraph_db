use crate::{ParseError, lexer::Lexeme, parser::{Ident, Parsable, Parser}};

#[derive(Debug, Clone)]
pub struct DeleteStmt {
    pub parts: Vec<DeletePart>
}
impl Parsable for DeleteStmt {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(Lexeme::Delete)?;
        let parts = DeletePart::parse_one_or_more(parser, Some(Lexeme::Comma))?;
        Ok(Self { parts: parts })
    }
}

#[derive(Debug, Clone)]
pub struct DeletePart {
    pub alias: String,
    pub attr_name: Option<String>,
}
impl Parsable for DeletePart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let alias: String = Ident::parse(parser)?.to_string();
        let attr_name = if let Some(_) = parser.expect_probably(Lexeme::Dot) {
            Some(Ident::parse(parser)?.to_string())
        } else {None};

        Ok(Self { alias, attr_name })
    }
}