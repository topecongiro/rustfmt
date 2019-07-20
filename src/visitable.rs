#![allow(unused)]

use syntax::ast;
use syntax::ptr::P;

use crate::spanned::Spanned;
use crate::utils::{semicolon_for_expr, stmt_expr};
use crate::visitor::FmtVisitor;
use crate::Config;

/// An AST node that can be visited by `FmtVisitor`. We use this trait to abstract the processing
/// of formatting block.
pub(crate) trait Visitable: Sized + Spanned {
    fn visit_on(visitor: &mut FmtVisitor<'_>, visitables: &[Self]);
    fn requires_semicolon(&self, config: &Config) -> bool;
}

impl Visitable for ast::Stmt {
    fn visit_on(visitor: &mut FmtVisitor<'_>, visitables: &[Self]) {
        visitor.walk_block_stmts(visitables);
    }

    fn requires_semicolon(&self, config: &Config) -> bool {
        stmt_expr(self).map_or(false, |expr| semicolon_for_expr(config, expr))
    }
}

impl Visitable for P<ast::Item> {
    fn visit_on(visitor: &mut FmtVisitor<'_>, visitables: &[Self]) {
        unimplemented!()
    }

    fn requires_semicolon(&self, _: &Config) -> bool {
        true
    }
}