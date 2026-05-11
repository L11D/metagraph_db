use parser::Stmt;

use crate::query::{
    QueryEngine, common::QueryContext, engine::Executable, error::EngineError
};

mod find;
mod create;
mod return_stmt;
mod set;
mod delete;
mod parent;

impl Executable for Stmt {
    fn execute(&self, qe: &QueryEngine, ctx: &mut QueryContext) -> Result<(), EngineError> {
        for find in &self.finds {
            find.execute(qe, ctx)?;
        }

        for create in &self.creates {
            create.execute(qe, ctx)?;
        }

        if let Some(set) = &self.set {
            set.execute(qe, ctx)?;
        }

        if let Some(parent) = &self.parent {
            parent.execute(qe, ctx)?;
        }

        if let Some(delete) = &self.delete {
            delete.execute(qe, ctx)?;
        }

        if let Some(return_stmt) = &self.return_stmt {
            return_stmt.execute(qe, ctx)?;
        }

        Ok(())
    }
}
