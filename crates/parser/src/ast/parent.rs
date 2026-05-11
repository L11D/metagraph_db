use crate::{ParseError, lexer::Lexeme, parser::{Ident, Parsable, Parser}};

// PARENT x TO y
#[derive(Debug, Clone)]
pub struct ParentStmt {
    pub child: String,
    pub parent: String,
    pub is_parenting: bool,
}
impl Parsable for ParentStmt {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let is_parenting = if let Some(_) = parser.expect_probably(Lexeme::Parent) {
            true
        } else {
            parser.expect(Lexeme::Unparent)?;
            false
        };

        let child = Ident::parse(parser)?.to_string();
        if let None = parser.expect_probably(Lexeme::To) {
            parser.expect(Lexeme::From)?;
        }
        let parent = Ident::parse(parser)?.to_string();

        Ok(Self { child, parent, is_parenting })
    }
}