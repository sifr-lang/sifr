use super::implementation::{AnalysisHost, QueryResult};
use crate::queries::{
    CodeAction, CodeActionContext, CompletionItems, DiagnosticExplanation, DiagnosticId,
    DocumentHighlight, DocumentSymbol, FileDiagnostics, FoldingRange, FormatOptions,
    GeneratedRustPreview, HoverInfo, InlayHint, Location, RenameTarget, SelectionRange,
    SemanticToken, SignatureHelp, SymbolName, SymbolQuery, TestCommand, TestItem, TestItemId,
    TypeHierarchyItem, TypeHierarchyItemId, WorkspaceEdit, WorkspaceSymbol,
};
use crate::snapshot::AnalysisSnapshot;
use ruff_text_size::TextRange;
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::FileId;
use sifr_syntax::TextPosition;

impl AnalysisSnapshot {
    fn run<T>(
        &self,
        host: &mut AnalysisHost,
        query: impl FnOnce(&mut AnalysisHost) -> QueryResult<T>,
    ) -> QueryResult<T> {
        host.ensure_snapshot_current(self)?;
        query(host).map(|result| result.with_workspace_snapshot_id(self.workspace_snapshot_id()))
    }

    pub fn diagnostics(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
    ) -> QueryResult<Vec<RenderedDiagnostic>> {
        self.run(host, |host| host.diagnostics(file))
    }

    pub fn workspace_diagnostics(
        &self,
        host: &mut AnalysisHost,
    ) -> QueryResult<Vec<FileDiagnostics>> {
        self.run(host, AnalysisHost::workspace_diagnostics)
    }

    pub fn completion(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<CompletionItems> {
        self.run(host, |host| host.completion(file, position))
    }

    pub fn hover(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Option<HoverInfo>> {
        self.run(host, |host| host.hover(file, position))
    }

    pub fn signature_help(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Option<SignatureHelp>> {
        self.run(host, |host| host.signature_help(file, position))
    }

    pub fn definition(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Vec<Location>> {
        self.run(host, |host| host.definition(file, position))
    }

    pub fn declaration(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Vec<Location>> {
        self.run(host, |host| host.declaration(file, position))
    }

    pub fn type_definition(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Vec<Location>> {
        self.run(host, |host| host.type_definition(file, position))
    }

    pub fn references(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Vec<Location>> {
        self.run(host, |host| host.references(file, position))
    }

    pub fn prepare_rename(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Option<RenameTarget>> {
        self.run(host, |host| host.prepare_rename(file, position))
    }

    pub fn rename(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        position: &TextPosition,
        new_name: &SymbolName,
    ) -> QueryResult<WorkspaceEdit> {
        self.run(host, |host| host.rename(file, position, new_name))
    }

    pub fn document_symbols(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
    ) -> QueryResult<Vec<DocumentSymbol>> {
        self.run(host, |host| host.document_symbols(file))
    }

    pub fn workspace_symbols(
        &self,
        host: &mut AnalysisHost,
        query: &SymbolQuery,
    ) -> QueryResult<Vec<WorkspaceSymbol>> {
        self.run(host, |host| host.workspace_symbols(query))
    }

    pub fn semantic_tokens(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        range: Option<TextRange>,
    ) -> QueryResult<Vec<SemanticToken>> {
        self.run(host, |host| host.semantic_tokens(file, range))
    }

    pub fn inlay_hints(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        range: Option<TextRange>,
    ) -> QueryResult<Vec<InlayHint>> {
        self.run(host, |host| host.inlay_hints(file, range))
    }

    pub fn document_highlights(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Vec<DocumentHighlight>> {
        self.run(host, |host| host.document_highlights(file, position))
    }

    pub fn folding_ranges(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
    ) -> QueryResult<Vec<FoldingRange>> {
        self.run(host, |host| host.folding_ranges(file))
    }

    pub fn selection_ranges(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        positions: &[TextPosition],
    ) -> QueryResult<Vec<SelectionRange>> {
        self.run(host, |host| host.selection_ranges(file, positions))
    }

    pub fn prepare_type_hierarchy(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Option<TypeHierarchyItem>> {
        self.run(host, |host| host.prepare_type_hierarchy(file, position))
    }

    pub fn type_hierarchy_supertypes(
        &self,
        host: &mut AnalysisHost,
        item: TypeHierarchyItemId,
    ) -> QueryResult<Vec<TypeHierarchyItem>> {
        self.run(host, |host| host.type_hierarchy_supertypes(item))
    }

    pub fn type_hierarchy_subtypes(
        &self,
        host: &mut AnalysisHost,
        item: TypeHierarchyItemId,
    ) -> QueryResult<Vec<TypeHierarchyItem>> {
        self.run(host, |host| host.type_hierarchy_subtypes(item))
    }

    pub fn code_actions(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        range: TextRange,
        context: &CodeActionContext,
    ) -> QueryResult<Vec<CodeAction>> {
        self.run(host, |host| host.code_actions(file, range, context))
    }

    pub fn safe_fix_all_action(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
    ) -> QueryResult<WorkspaceEdit> {
        self.run(host, |host| host.safe_fix_all_action(file))
    }

    pub fn format_document(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        options: FormatOptions,
    ) -> QueryResult<Vec<sifr_format::TextEdit>> {
        self.run(host, |host| host.format_document(file, options))
    }

    pub fn format_range(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        range: TextRange,
        options: FormatOptions,
    ) -> QueryResult<Vec<sifr_format::TextEdit>> {
        self.run(host, |host| host.format_range(file, range, options))
    }

    pub fn generated_rust_preview(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
        range: Option<TextRange>,
    ) -> QueryResult<GeneratedRustPreview> {
        self.run(host, |host| host.generated_rust_preview(file, range))
    }

    pub fn explain_diagnostic(
        &self,
        host: &mut AnalysisHost,
        diagnostic: &DiagnosticId,
    ) -> QueryResult<DiagnosticExplanation> {
        self.run(host, |host| host.explain_diagnostic(diagnostic))
    }

    pub fn discover_tests(&self, host: &mut AnalysisHost) -> QueryResult<Vec<TestItem>> {
        self.run(host, AnalysisHost::discover_tests)
    }

    pub fn test_command(
        &self,
        host: &mut AnalysisHost,
        test: TestItemId,
    ) -> QueryResult<TestCommand> {
        self.run(host, |host| host.test_command(test))
    }
}
