#![allow(unused)]

use syntax::ast;
use syntax::ptr::P;

use crate::expr::stmt_is_expr;
use crate::spanned::Spanned;
use crate::utils::{ptr_vec_to_ref_vec, semicolon_for_expr, stmt_expr};
use crate::visitor::FmtVisitor;
use crate::Config;

/// An AST node that can be visited by `FmtVisitor`. We use this trait to abstract the processing
/// of formatting block.
pub(crate) trait Visitable: Sized + Spanned {
    fn visit_on(visitor: &mut FmtVisitor<'_>, visitables: &[Self]);
    fn requires_semicolon(&self, config: &Config) -> bool;
    fn can_be_single_lined(&self) -> bool;
}

impl Visitable for ast::Stmt {
    fn visit_on(visitor: &mut FmtVisitor<'_>, visitables: &[Self]) {
        visitor.walk_block_stmts(visitables);
    }

    fn requires_semicolon(&self, config: &Config) -> bool {
        stmt_expr(self).map_or(false, |expr| semicolon_for_expr(config, expr))
    }

    fn can_be_single_lined(&self) -> bool {
        stmt_is_expr(self)
    }
}

impl Visitable for P<ast::Item> {
    fn visit_on(visitor: &mut FmtVisitor<'_>, visitables: &[Self]) {
        visitor.visit_items_with_reordering(&ptr_vec_to_ref_vec(&visitables));
    }

    fn requires_semicolon(&self, _: &Config) -> bool {
        false
    }

    fn can_be_single_lined(&self) -> bool {
        false
    }
}
