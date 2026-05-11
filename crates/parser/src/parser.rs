use crate::{ParseError, lexer::Lexeme};

pub struct Parser<'a> {
    lexemes: &'a [Lexeme],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(lexemes: &'a [Lexeme]) -> Self {
        Self { lexemes, pos: 0 }
    }

    pub fn peek(&self) -> Option<&Lexeme> {
        self.lexemes.get(self.pos)
    }

    pub fn next(&mut self) -> Option<Lexeme> {
        let res = self.peek().cloned();
        if res.is_some() {
            self.pos += 1;
        }
        res
    }

    // Сохраняем состояние для отката
    pub fn checkpoint(&self) -> usize {
        self.pos
    }

    pub fn rollback(&mut self, checkpoint: usize) {
        self.pos = checkpoint;
    }

    pub fn expect(&mut self, expected: Lexeme) -> Result<(), ParseError> {
        match self.peek() {
            Some(l) => {
                if *l == expected {
                    self.next();
                    Ok(())
                } else {
                    Err(ParseError::UnexpectedNode(
                        expected.to_string(),
                        l.to_string(),
                    ))
                }
            }
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    pub fn expect_probably(&mut self, expected: Lexeme) -> Option<()> {
        let cp = self.checkpoint();
        match self.expect(expected) {
            Ok(p) => Some(p),
            Err(_) => {
                self.rollback(cp);
                None
            }
        }
    }
}



pub trait Parsable: Sized {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError>;

    fn parse_one_or_more(
        parser: &mut Parser,
        separator: Option<Lexeme>,
    ) -> Result<Vec<Self>, ParseError> {
        let mut result = vec![Self::parse(parser)?];

        match separator {
            None => result.extend(Self::parse_any_count(parser, None)),

            Some(sep) => {
                if parser.expect_probably(sep.clone()).is_some() {
                    result.extend(Self::parse_any_count(parser, Some(sep)))
                }
            }
        }

        Ok(result)
    }

    fn parse_any_count(parser: &mut Parser, separator: Option<Lexeme>) -> Vec<Self> {
        let mut result = Vec::new();

        let Some(first) = Self::parse_probably(parser) else {
            return result;
        };
        result.push(first);

        match separator {
            None => {
                while let Some(item) = Self::parse_probably(parser) {
                    result.push(item);
                }
            }

            Some(sep) => {
                while parser.expect_probably(sep.clone()).is_some() {
                    let Some(item) = Self::parse_probably(parser) else {
                        break;
                    };
                    result.push(item);
                }
            }
        }

        result
    }

    fn parse_probably(parser: &mut Parser) -> Option<Self> {
        let cp = parser.checkpoint();
        match Self::parse(parser) {
            Ok(p) => Some(p),
            Err(_) => {
                parser.rollback(cp);
                None
            }
        }
    }
}

pub trait OptionToString {
    fn to_string(self) -> Option<String>;
}

pub struct Ident(String);
impl Ident {
    pub fn to_string(self) -> String {
        self.0
    }
}
impl OptionToString for Option<Ident> {
    fn to_string(self) -> Option<String> {
        self.map(|ident| ident.0)
    }
}
impl Parsable for Ident {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek() {
            Some(Lexeme::Ident(_)) => {
                let Lexeme::Ident(s) = parser.next().unwrap() else {
                    unreachable!()
                };
                Ok(Ident(s))
            }
            Some(l) => Err(ParseError::UnexpectedNode(
                "Identifier".into(),
                l.to_string(),
            )),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}

impl Parsable for String {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek() {
            Some(Lexeme::String(_)) => {
                let Lexeme::String(s) = parser.next().unwrap() else {
                    unreachable!()
                };
                Ok(s)
            }
            Some(l) => Err(ParseError::UnexpectedNode("String".into(), l.to_string())),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}

impl Parsable for i64 {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek() {
            Some(Lexeme::Number(n)) => {
                let n = *n;
                parser.next();
                Ok(n)
            }
            Some(l) => Err(ParseError::UnexpectedNode("Number".into(), l.to_string())),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}

impl Parsable for bool {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek() {
            Some(Lexeme::Bool(b)) => {
                let b = *b;
                parser.next();
                Ok(b)
            }
            Some(l) => Err(ParseError::UnexpectedNode("Bool".into(), l.to_string())),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    #[test]
    fn test() -> Result<(), ParseError> {
        let input = r#"
        CREATE a
    "#;

        let lexemes = lexer::parse(input).unwrap();
        let mut parser = Parser::new(&lexemes);

        parser.expect(Lexeme::Create)?;
        // // let ident = parser.expect_string();
        // match ident {
        //     Ok(_) => todo!(),
        //     Err(e) => println!("{}", e),
        // }

        Ok(())
    }
}
