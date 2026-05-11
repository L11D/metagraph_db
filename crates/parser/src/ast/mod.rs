mod common;
mod find;
mod create;
mod return_stmt;
mod set;
mod parent;
mod filter;
mod delete;

pub use common::*;
pub use filter::*;
pub use find::*;
pub use create::*; 
pub use return_stmt::*;
pub use set::*;
pub use parent::*;
pub use delete::*;

use crate::{ParseError, parser::Parsable};

#[derive(Debug, Clone)]
pub struct Stmt {
    pub finds: Vec<FindStmt>,
    pub creates: Vec<CreateStmt>,
    pub set: Option<SetStmt>,
    pub parent: Option<ParentStmt>,
    pub delete: Option<DeleteStmt>,
    pub return_stmt: Option<ReturnStmt>,
}
impl Parsable for Stmt {
    fn parse(parser: &mut crate::parser::Parser) -> Result<Self, ParseError> {
        let mut finds = FindStmt::parse_any_count(parser, None);
        let mut creates = CreateStmt::parse_any_count(parser, None);

        if finds.is_empty() && creates.is_empty() {
            return Err(ParseError::MissFindOrCreate)
        }

        let mut set = None;
        let mut parent = None;
        let mut delete = None;
        let mut return_stmt = None;

        loop {
            let mut smth_parsed = false;

            let in_finds = FindStmt::parse_any_count(parser, None);
            if !in_finds.is_empty() {
                finds.extend(in_finds);
                smth_parsed = true;
            }

            let in_creates = CreateStmt::parse_any_count(parser, None);
            if !in_creates.is_empty() {
                creates.extend(in_creates);
                smth_parsed = true;
            }

            if let Some(s) = SetStmt::parse_probably(parser) {
                smth_parsed = true;
                if set.is_none() {
                    set = Some(s);
                } else {
                    return Err(ParseError::AlreadyDefined("set".into()));
                }
            }

            if let Some(p) = ParentStmt::parse_probably(parser) {
                smth_parsed = true;
                if parent.is_none() {
                    parent = Some(p);
                } else {
                    return Err(ParseError::AlreadyDefined("parent".into()));
                }
            }

            if let Some(d) = DeleteStmt::parse_probably(parser) {
                smth_parsed = true;
                if delete.is_none() {
                    delete = Some(d);
                } else {
                    return Err(ParseError::AlreadyDefined("delete".into()));
                }
            }

            if let Some(r) = ReturnStmt::parse_probably(parser) {
                smth_parsed = true;
                if return_stmt.is_none() {
                    return_stmt = Some(r);
                } else {
                    return Err(ParseError::AlreadyDefined("return".into()));
                }
            }

            if !smth_parsed {
                break;
            }
        };

        Ok(Self {
            finds,
            creates,
            set: set,
            parent: parent,
            delete: delete,
            return_stmt: return_stmt
        })
    }
}