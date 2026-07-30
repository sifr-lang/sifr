use super::{sqlx_query_text, SqlxQueryVisitor};
use crate::build::rust_interop_sqlx_modules::has_conditional_compilation_attribute;
use syn::visit::Visit;
use syn::{Attribute, Expr, ForeignItem, ImplItem, Item, Macro, Stmt, TraitItem};

impl<'ast> Visit<'ast> for SqlxQueryVisitor<'_> {
    fn visit_item(&mut self, node: &'ast Item) {
        if !item_has_cfg_attribute(node) {
            syn::visit::visit_item(self, node);
        }
    }

    fn visit_stmt(&mut self, node: &'ast Stmt) {
        if !stmt_has_cfg_attribute(node) {
            syn::visit::visit_stmt(self, node);
        }
    }

    fn visit_expr(&mut self, node: &'ast Expr) {
        if !expr_has_cfg_attribute(node) {
            syn::visit::visit_expr(self, node);
        }
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        if !has_cfg_attribute(&node.attrs) {
            syn::visit::visit_arm(self, node);
        }
    }

    fn visit_field(&mut self, node: &'ast syn::Field) {
        if !has_cfg_attribute(&node.attrs) {
            syn::visit::visit_field(self, node);
        }
    }

    fn visit_field_value(&mut self, node: &'ast syn::FieldValue) {
        if !has_cfg_attribute(&node.attrs) {
            syn::visit::visit_field_value(self, node);
        }
    }

    fn visit_variant(&mut self, node: &'ast syn::Variant) {
        if !has_cfg_attribute(&node.attrs) {
            syn::visit::visit_variant(self, node);
        }
    }

    fn visit_generic_param(&mut self, node: &'ast syn::GenericParam) {
        if !generic_param_has_cfg_attribute(node) {
            syn::visit::visit_generic_param(self, node);
        }
    }

    fn visit_fn_arg(&mut self, node: &'ast syn::FnArg) {
        let attrs = match node {
            syn::FnArg::Receiver(receiver) => &receiver.attrs,
            syn::FnArg::Typed(argument) => &argument.attrs,
        };
        if !has_cfg_attribute(attrs) {
            syn::visit::visit_fn_arg(self, node);
        }
    }

    fn visit_impl_item(&mut self, node: &'ast ImplItem) {
        if !impl_item_has_cfg_attribute(node) {
            syn::visit::visit_impl_item(self, node);
        }
    }

    fn visit_trait_item(&mut self, node: &'ast TraitItem) {
        if !trait_item_has_cfg_attribute(node) {
            syn::visit::visit_trait_item(self, node);
        }
    }

    fn visit_foreign_item(&mut self, node: &'ast ForeignItem) {
        if !foreign_item_has_cfg_attribute(node) {
            syn::visit::visit_foreign_item(self, node);
        }
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        if let Some(query) = sqlx_query_text(node, self.aliases, self.backend_root) {
            self.queries.push(query);
        }
        syn::visit::visit_macro(self, node);
    }

    fn visit_item_mod(&mut self, _node: &'ast syn::ItemMod) {}
}

pub(super) fn has_cfg_attribute(attrs: &[Attribute]) -> bool {
    has_conditional_compilation_attribute(attrs)
}

pub(super) fn item_has_cfg_attribute(item: &Item) -> bool {
    let attrs = match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        Item::Verbatim(_) => return false,
        _ => return true,
    };
    has_cfg_attribute(attrs)
}

fn stmt_has_cfg_attribute(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Local(local) => has_cfg_attribute(&local.attrs),
        Stmt::Item(item) => item_has_cfg_attribute(item),
        Stmt::Expr(expr, _) => expr_has_cfg_attribute(expr),
        Stmt::Macro(stmt) => has_cfg_attribute(&stmt.attrs),
    }
}

fn expr_has_cfg_attribute(expr: &Expr) -> bool {
    let attrs = match expr {
        Expr::Array(expr) => &expr.attrs,
        Expr::Assign(expr) => &expr.attrs,
        Expr::Async(expr) => &expr.attrs,
        Expr::Await(expr) => &expr.attrs,
        Expr::Binary(expr) => &expr.attrs,
        Expr::Block(expr) => &expr.attrs,
        Expr::Break(expr) => &expr.attrs,
        Expr::Call(expr) => &expr.attrs,
        Expr::Cast(expr) => &expr.attrs,
        Expr::Closure(expr) => &expr.attrs,
        Expr::Const(expr) => &expr.attrs,
        Expr::Continue(expr) => &expr.attrs,
        Expr::Field(expr) => &expr.attrs,
        Expr::ForLoop(expr) => &expr.attrs,
        Expr::Group(expr) => &expr.attrs,
        Expr::If(expr) => &expr.attrs,
        Expr::Index(expr) => &expr.attrs,
        Expr::Infer(expr) => &expr.attrs,
        Expr::Let(expr) => &expr.attrs,
        Expr::Lit(expr) => &expr.attrs,
        Expr::Loop(expr) => &expr.attrs,
        Expr::Macro(expr) => &expr.attrs,
        Expr::Match(expr) => &expr.attrs,
        Expr::MethodCall(expr) => &expr.attrs,
        Expr::Paren(expr) => &expr.attrs,
        Expr::Path(expr) => &expr.attrs,
        Expr::Range(expr) => &expr.attrs,
        Expr::RawAddr(expr) => &expr.attrs,
        Expr::Reference(expr) => &expr.attrs,
        Expr::Repeat(expr) => &expr.attrs,
        Expr::Return(expr) => &expr.attrs,
        Expr::Struct(expr) => &expr.attrs,
        Expr::Try(expr) => &expr.attrs,
        Expr::TryBlock(expr) => &expr.attrs,
        Expr::Tuple(expr) => &expr.attrs,
        Expr::Unary(expr) => &expr.attrs,
        Expr::Unsafe(expr) => &expr.attrs,
        Expr::Verbatim(_) => return false,
        Expr::While(expr) => &expr.attrs,
        Expr::Yield(expr) => &expr.attrs,
        _ => return true,
    };
    has_cfg_attribute(attrs)
}

fn impl_item_has_cfg_attribute(item: &ImplItem) -> bool {
    let attrs = match item {
        ImplItem::Const(item) => &item.attrs,
        ImplItem::Fn(item) => &item.attrs,
        ImplItem::Type(item) => &item.attrs,
        ImplItem::Macro(item) => &item.attrs,
        ImplItem::Verbatim(_) => return false,
        _ => return true,
    };
    has_cfg_attribute(attrs)
}

fn trait_item_has_cfg_attribute(item: &TraitItem) -> bool {
    let attrs = match item {
        TraitItem::Const(item) => &item.attrs,
        TraitItem::Fn(item) => &item.attrs,
        TraitItem::Type(item) => &item.attrs,
        TraitItem::Macro(item) => &item.attrs,
        TraitItem::Verbatim(_) => return false,
        _ => return true,
    };
    has_cfg_attribute(attrs)
}

fn foreign_item_has_cfg_attribute(item: &ForeignItem) -> bool {
    let attrs = match item {
        ForeignItem::Fn(item) => &item.attrs,
        ForeignItem::Static(item) => &item.attrs,
        ForeignItem::Type(item) => &item.attrs,
        ForeignItem::Macro(item) => &item.attrs,
        ForeignItem::Verbatim(_) => return false,
        _ => return true,
    };
    has_cfg_attribute(attrs)
}

fn generic_param_has_cfg_attribute(param: &syn::GenericParam) -> bool {
    let attrs = match param {
        syn::GenericParam::Lifetime(param) => &param.attrs,
        syn::GenericParam::Type(param) => &param.attrs,
        syn::GenericParam::Const(param) => &param.attrs,
    };
    has_cfg_attribute(attrs)
}
