mod lexer;
mod ast;
mod parser;
mod error;

use log::LevelFilter;
use parser::*;
use lexer::*;

pub use error::*;
pub use ast::*;
use simple_logger::SimpleLogger;



pub fn parse(input: &str) -> Result<Stmt, ParseError> {
    // SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    let lexemes = lexer::parse(input)?;
    let mut parser = parser::Parser::new(&lexemes);
    let result = Stmt::parse(&mut parser)?;
    parser.expect(Lexeme::EOF)?;
    // log::info!("{:#?}", result);
    Ok(result)
}


#[cfg(test)]
mod tests{
    use crate::parse;


    #[test]
    fn test() {
        let input = r#"
        FIND (a in dog)
        "#;
        let result = parse(input);
        match result {
            Ok(_) => println!("OK"),
            Err(e) => println!("{:#?}", e),
        }
    }
}