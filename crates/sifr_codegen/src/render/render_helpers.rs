use super::{Renderer, RustExpr, RustFile, RustItem, RustStmt, RustType};

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_items(items: &[RustItem]) -> String {
    let file = RustFile {
        items: items.to_vec(),
    };
    Renderer::new().render_file(&file)
}

pub fn render_stmts(stmts: &[RustStmt]) -> String {
    let mut renderer = Renderer::new();
    for stmt in stmts {
        renderer.render_stmt(stmt);
    }
    renderer.output
}

pub fn render_expr(expr: &RustExpr) -> String {
    let mut renderer = Renderer::new();
    renderer.render_expr(expr);
    renderer.output
}

pub fn render_type(ty: &RustType) -> String {
    let mut renderer = Renderer::new();
    renderer.render_type(ty);
    renderer.output
}

#[cfg(test)]
#[path = "../../tests/unit/render/render_helpers/tests.rs"]
mod tests;
