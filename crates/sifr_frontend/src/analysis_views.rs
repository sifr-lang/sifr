use crate::{EditorSemanticView, ModuleId, SqlEditorDocumentView};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolView {
    pub name: String,
    pub kind: SymbolKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Class,
    Constant,
    Import,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassHierarchyView {
    pub name: String,
    pub identity: String,
    pub parent_identity: Option<String>,
    pub parent_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleAnalysisView {
    pub module: ModuleId,
    pub symbols: Vec<SymbolView>,
    pub classes: Vec<ClassHierarchyView>,
    pub editor_semantics: EditorSemanticView,
    pub sql_documents: Vec<SqlEditorDocumentView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectAnalysisView {
    pub modules: Vec<ModuleAnalysisView>,
}
