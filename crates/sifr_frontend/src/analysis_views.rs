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
pub struct ModuleAnalysisView {
    pub module: ModuleId,
    pub symbols: Vec<SymbolView>,
    pub editor_semantics: EditorSemanticView,
    pub sql_documents: Vec<SqlEditorDocumentView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectAnalysisView {
    pub modules: Vec<ModuleAnalysisView>,
}
