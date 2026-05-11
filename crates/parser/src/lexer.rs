use std::fmt;

use logos::{Lexer, Logos, Span};
use thiserror::Error;

fn parse_string(lex: &mut Lexer<Lexeme>) -> Option<String> {
    let slice = lex.slice();
    Some(slice[1..slice.len() - 1].to_string())
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")] // Игнорируем пробелы и переносы
pub enum Lexeme {
    // Ключевые слова (регистрозависимые)
    #[token("CREATE",)]
    #[token("create")]
    Create,

    #[token("META")]
    #[token("meta")]
    Meta,

    #[token("VERT")]
    #[token("vert")]
    Vert,

    #[token("EDGE")]
    #[token("edge")]
    Edge,

    #[token("FIND")]
    #[token("find")]
    Find,

    #[token("DELETE")]
    #[token("delete")]
    Delete,

    #[token("WHERE")]
    #[token("where")]
    Where,

    #[token("CONSTRAINT")]
    #[token("constraint")]
    Constraint,
    
    #[token("ATTRIBUTE")]
    #[token("attribute")]
    Attribute,

    #[token("RETURN")]
    #[token("return")]
    Return,

    #[token("SET")]
    #[token("set")]
    Set,

    #[token("PARENT")]
    #[token("parent")]
    Parent,

    #[token("UNPARENT")]
    #[token("unparent")]
    Unparent,

    #[token("IN")]
    #[token("in")]
    In,

    #[token("TO")]
    #[token("to")]
    To,

    #[token("FROM")]
    #[token("from")]
    From,

    #[token("AND")]
    #[token("and")]
    And,

    #[token("OR")]
    #[token("or")]
    Or,

    #[token("NOT")]
    #[token("not")]
    Not,

    // Типы
    #[token("text")]
    TypeText,
    #[token("int32")]
    TypeInt32,
    #[token("bool")]
    TypeBool,

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Number(i64),

    #[regex("true|false", |lex| match lex.slice() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(()),
    })]
    Bool(bool),

    #[token("None")]
    #[token("none")]
    #[token("Null")]
    #[token("null")]
    Null,

    // CompOp
    #[token("!=")]
    NotEqual,
    #[token("<=")]
    LessOrEqual,
    #[token(">=")]
    GreaterOrEqual,
    #[token("==")]
    #[token("=")]
    Equal,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,

    // Символы и операторы

    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("?")]
    Question,
    #[token("-")]
    Dash,

    // Идентификаторы
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Литералы
    #[regex(r#""([^"\\]|\\.)*""#, parse_string)]
    // #[regex(r"[\p{L}_][\p{L}\p{N}_# № ]*", parse_string)]
    String(String),

    EOF
}

impl fmt::Display for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lexeme::Ident(s) => write!(f, "{}", s),
            Lexeme::Number(_) => write!(f, "number"),
            Lexeme::String(_) => write!(f, "string"),
            Lexeme::Bool(_) => write!(f, "bool"),
            // ... остальные варианты
            _ => write!(f, "{:?}", self), // фоллбэк на Debug
        }
    }
}

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unknown lexeme at position: {0:?}")]
    UnknownLexeme(Span)
}

pub fn parse(input: &str) -> Result<Vec<Lexeme>, LexerError> {
    let mut result = Vec::with_capacity(25);
    let mut lexer = Lexeme::lexer(input);
    while let Some(token) = lexer.next() {
        match token {
            Ok(t) => result.push(t),
            Err(_) => return Err(LexerError::UnknownLexeme(lexer.span())),
        }
    }
    result.push(Lexeme::EOF);
    Ok(result)
}

#[test]
fn test() {
    let input = r#"
    CREATE a:VERT {
        name: text = "Лизочка",
        age: int32 = 7,
        sсhool: text = "Школа1"
    }
    RETURN a
    "#;

    let result = parse(input);

    match result {
        Ok(r) => println!("{:#?}, {}", r, r.len()),
        Err(e) => println!("{}", e),
    }
}