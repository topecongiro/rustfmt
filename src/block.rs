#![allow(unused)]

use std::marker::PhantomData;

use syntax::ast;
use syntax::ptr::P;
use syntax_pos::{BytePos, Pos, Span};

use crate::comment::{contains_comment, rewrite_comment, CodeCharKind, CommentCodeSlices};
use crate::coverage::transform_missing_snippet;
use crate::items::is_use_item;
use crate::rewrite::RewriteContext;
use crate::shape::Shape;
use crate::source_map::LineRangeUtils;
use crate::spanned::Spanned;
use crate::stmt::Stmt;
use crate::utils::{
    count_newlines, inner_attributes, last_line_width, mk_sp, semicolon_for_expr, stmt_expr,
};
use crate::visitable::Visitable;
use crate::visitor::FmtVisitor;

const BRACE_COMPENSATION: BytePos = BytePos(1);

/// A block (`{ ... }`). Its items can be visited by `FmtVisitor`.
pub(crate) struct Block<'a, T> {
    items: &'a [T],
    inner_attrs: Option<&'a [ast::Attribute]>,
    empty_block_style: EmptyBlockStyle,
    span: Span,
}

#[derive(Copy, Clone)]
pub(crate) enum EmptyBlockStyle {
    /// Allow putting an empty block in a single line.
    SingleLine,
    /// Forbid putting an empty block in a single line.
    MultiLine,
}

impl<'a> Block<'a, ast::Stmt> {
    pub(crate) fn from_ast_block(
        block: &'a ast::Block,
        inner_attrs: Option<&'a [ast::Attribute]>,
        empty_block_style: EmptyBlockStyle,
    ) -> Self {
        Block {
            items: block.stmts.as_slice(),
            inner_attrs,
            empty_block_style,
            span: block.span,
        }
    }
}

impl<'a> Block<'a, P<ast::Item>> {
    pub(crate) fn from_ast_module(
        module: &'a ast::Mod,
        inner_attrs: &'a [ast::Attribute],
        empty_block_style: EmptyBlockStyle,
    ) -> Self {
        debug_assert!(module.inline);
        Block {
            items: module.items.as_slice(),
            inner_attrs: if inner_attrs.is_empty() {
                None
            } else {
                Some(inner_attrs)
            },
            empty_block_style,
            span: module.inner,
        }
    }
}

impl<'b, 'a: 'b> FmtVisitor<'a> {
    fn is_empty_block<T>(&self, block: &Block<'_, T>) -> bool {
        block.items.is_empty()
            && block.inner_attrs.map_or(true, |attrs| attrs.is_empty())
            && !contains_comment(self.snippet(block.span))
    }

    pub(crate) fn visit_block<T: Visitable>(&mut self, b: &Block<'_, T>) {
        debug!(
            "visit_block: {:?} {:?}",
            self.source_map.lookup_char_pos(b.span.lo()),
            self.source_map.lookup_char_pos(b.span.hi())
        );

        self.last_pos = self.last_pos + BRACE_COMPENSATION;
        self.block_indent = self.block_indent.block_indent(self.config);
        self.push_str("{");

        if self.is_empty_block(b) {
            self.block_indent = self.block_indent.block_unindent(self.config);
            match b.empty_block_style {
                EmptyBlockStyle::SingleLine
                    if last_line_width(&self.buffer) < self.config.max_width() =>
                {
                    self.push_str("}");
                }
                _ => {
                    self.push_str(&self.block_indent.to_string_with_newline(self.config));
                    self.push_str("}");
                }
            }
            self.last_pos = source!(self, b.span).hi();
            return;
        }

        self.trim_spaces_after_opening_brace(b, b.inner_attrs);

        // Format inner attributes if available.
        if let Some(attrs) = b.inner_attrs {
            self.visit_attrs(attrs, ast::AttrStyle::Inner);
        }

        Visitable::visit_on(self, b.items);

        // TODO: This should be handled within each visit method.
        if let Some(last_item) = b.items.last() {
            if last_item.requires_semicolon(self.config) {
                self.push_str(";");
            }
        }

        let rest_span = self.next_span(b.span.hi());
        if out_of_file_lines_range!(self, rest_span) {
            self.push_str(self.snippet(rest_span));
            self.block_indent = self.block_indent.block_unindent(self.config);
        } else {
            // Ignore the closing brace.
            let missing_span = self.next_span(b.span.hi() - BRACE_COMPENSATION);
            self.close_block(missing_span, self.unindent_comment_on_closing_brace(b));
        }
        self.last_pos = source!(self, b.span).hi();
    }

    /// Remove spaces between the opening brace and the first statement or the inner attribute
    /// of the block.
    fn trim_spaces_after_opening_brace<T: Visitable + Spanned>(
        &mut self,
        b: &Block<'_, T>,
        inner_attrs: Option<&[ast::Attribute]>,
    ) {
        if let Some(first_item) = b.items.first() {
            let hi = inner_attrs
                .and_then(|attrs| inner_attributes(attrs).first().map(|attr| attr.span.lo()))
                .unwrap_or_else(|| first_item.span().lo());
            let missing_span = self.next_span(hi);
            let snippet = self.snippet(missing_span);
            let len = CommentCodeSlices::new(snippet)
                .nth(0)
                .and_then(|(kind, _, s)| {
                    if kind == CodeCharKind::Normal {
                        s.rfind('\n')
                    } else {
                        None
                    }
                });
            if let Some(len) = len {
                self.last_pos = self.last_pos + BytePos::from_usize(len);
            }
        }
    }

    pub(crate) fn close_block(&mut self, span: Span, unindent_comment: bool) {
        let config = self.config;

        let mut last_hi = span.lo();
        let mut unindented = false;
        let mut prev_ends_with_newline = false;
        let mut extra_newline = false;

        let skip_normal = |s: &str| {
            let trimmed = s.trim();
            trimmed.is_empty() || trimmed.chars().all(|c| c == ';')
        };

        for (kind, offset, sub_slice) in CommentCodeSlices::new(self.snippet(span)) {
            let sub_slice = transform_missing_snippet(config, sub_slice);

            debug!("close_block: {:?} {:?} {:?}", kind, offset, sub_slice);

            match kind {
                CodeCharKind::Comment => {
                    if !unindented && unindent_comment {
                        unindented = true;
                        self.block_indent = self.block_indent.block_unindent(config);
                    }
                    let span_in_between = mk_sp(last_hi, span.lo() + BytePos::from_usize(offset));
                    let snippet_in_between = self.snippet(span_in_between);
                    let mut comment_on_same_line = !snippet_in_between.contains("\n");

                    let mut comment_shape =
                        Shape::indented(self.block_indent, config).comment(config);
                    if comment_on_same_line {
                        // 1 = a space before `//`
                        let offset_len = 1 + last_line_width(&self.buffer)
                            .saturating_sub(self.block_indent.width());
                        match comment_shape
                            .visual_indent(offset_len)
                            .sub_width(offset_len)
                        {
                            Some(shp) => comment_shape = shp,
                            None => comment_on_same_line = false,
                        }
                    };

                    if comment_on_same_line {
                        self.push_str(" ");
                    } else {
                        if count_newlines(snippet_in_between) >= 2 || extra_newline {
                            self.push_str("\n");
                        }
                        self.push_str(&self.block_indent.to_string_with_newline(config));
                    }

                    let comment_str = rewrite_comment(&sub_slice, false, comment_shape, config);
                    match comment_str {
                        Some(ref s) => self.push_str(s),
                        None => self.push_str(&sub_slice),
                    }
                }
                CodeCharKind::Normal if skip_normal(&sub_slice) => {
                    extra_newline = prev_ends_with_newline && sub_slice.contains('\n');
                    continue;
                }
                CodeCharKind::Normal => {
                    self.push_str(&self.block_indent.to_string_with_newline(config));
                    self.push_str(sub_slice.trim());
                }
            }
            prev_ends_with_newline = sub_slice.ends_with('\n');
            extra_newline = false;
            last_hi = span.lo() + BytePos::from_usize(offset + sub_slice.len());
        }
        if unindented {
            self.block_indent = self.block_indent.block_indent(self.config);
        }
        self.block_indent = self.block_indent.block_unindent(self.config);
        self.push_str(&self.block_indent.to_string_with_newline(config));
        self.push_str("}");
    }

    fn walk_stmts(&mut self, stmts: &[Stmt<'_>]) {
        if stmts.is_empty() {
            return;
        }

        // Extract leading `use ...;`.
        let items: Vec<_> = stmts
            .iter()
            .take_while(|stmt| stmt.to_item().map_or(false, is_use_item))
            .filter_map(|stmt| stmt.to_item())
            .collect();

        if items.is_empty() {
            self.visit_stmt(&stmts[0]);
            self.walk_stmts(&stmts[1..]);
        } else {
            self.visit_items_with_reordering(&items);
            self.walk_stmts(&stmts[items.len()..]);
        }
    }

    pub(crate) fn walk_block_stmts(&mut self, stmts: &[ast::Stmt]) {
        self.walk_stmts(&Stmt::from_ast_nodes(stmts.iter()))
    }

    fn unindent_comment_on_closing_brace<T>(&self, b: &Block<'_, T>) -> bool {
        self.is_if_else_block && !b.items.is_empty()
    }
}
