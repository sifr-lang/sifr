use super::implementation::{AnalysisHost, QueryResult};
use crate::queries::{Location, TypeHierarchyItem, TypeHierarchyItemId};
use crate::snapshot::{AnalysisError, AnalysisQueryKind};
use ruff_text_size::{Ranged, TextRange, TextSize};
use sifr_frontend::{ClassHierarchyView, FileId};
use sifr_python_ast::{Expr, Stmt};
use sifr_syntax::{SourceText as SyntaxSourceText, TextPosition};

struct ClassNode {
    item: TypeHierarchyItem,
    identity: String,
    parent_identity: Option<String>,
    parent_range: Option<TextRange>,
}

impl AnalysisHost {
    pub fn prepare_type_hierarchy(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Option<TypeHierarchyItem>> {
        let source = self.source_text(file)?;
        let Some(offset) = SyntaxSourceText::new(source).byte_offset(position) else {
            return Ok(self.result(AnalysisQueryKind::PrepareTypeHierarchy, None));
        };
        if !self.hierarchy_position_candidate(file, offset)? {
            return Ok(self.result(AnalysisQueryKind::PrepareTypeHierarchy, None));
        }
        let nodes = self.hierarchy_nodes()?;
        let selected = nodes
            .iter()
            .find(|node| {
                node.item.location.file == file
                    && node
                        .item
                        .location
                        .range
                        .is_some_and(|range| contains(range, offset))
            })
            .or_else(|| {
                nodes
                    .iter()
                    .find(|node| {
                        node.item.location.file == file
                            && node
                                .parent_range
                                .is_some_and(|range| contains(range, offset))
                    })
                    .and_then(|node| {
                        node.parent_identity.as_ref().and_then(|identity| {
                            nodes
                                .iter()
                                .find(|candidate| candidate.identity == *identity)
                        })
                    })
            });
        let item =
            selected.and_then(|node| has_hierarchy_edge(node, &nodes).then(|| node.item.clone()));
        Ok(self.result(AnalysisQueryKind::PrepareTypeHierarchy, item))
    }

    pub fn type_hierarchy_supertypes(
        &mut self,
        item: TypeHierarchyItemId,
    ) -> QueryResult<Vec<TypeHierarchyItem>> {
        let nodes = self.hierarchy_nodes()?;
        let parent = nodes
            .iter()
            .find(|node| node.item.id == item)
            .and_then(|node| node.parent_identity.as_ref())
            .and_then(|identity| {
                nodes
                    .iter()
                    .find(|candidate| candidate.identity == *identity)
            })
            .map(|node| node.item.clone());
        Ok(self.result(
            AnalysisQueryKind::TypeHierarchySupertypes,
            parent.into_iter().collect(),
        ))
    }

    pub fn type_hierarchy_subtypes(
        &mut self,
        item: TypeHierarchyItemId,
    ) -> QueryResult<Vec<TypeHierarchyItem>> {
        let nodes = self.hierarchy_nodes()?;
        let Some(parent) = nodes.iter().find(|node| node.item.id == item) else {
            return Ok(self.result(AnalysisQueryKind::TypeHierarchySubtypes, Vec::new()));
        };
        let children = nodes
            .iter()
            .filter(|node| node.parent_identity.as_deref() == Some(parent.identity.as_str()))
            .map(|node| node.item.clone())
            .collect();
        Ok(self.result(AnalysisQueryKind::TypeHierarchySubtypes, children))
    }

    fn hierarchy_position_candidate(
        &mut self,
        file: FileId,
        offset: TextSize,
    ) -> Result<bool, AnalysisError> {
        let Some(module) = self.file_to_module.get(&file).copied() else {
            return Ok(false);
        };
        let parsed = self.context_mut()?.parse_module(module).into_value().parsed;
        Ok(parsed.suite().iter().any(|statement| {
            let Stmt::ClassDef(class) = statement else {
                return false;
            };
            contains(class.name.range(), offset)
                || class.arguments.as_ref().is_some_and(|arguments| {
                    arguments
                        .args
                        .iter()
                        .any(|base| contains(base.range(), offset))
                })
        }))
    }

    fn hierarchy_nodes(&mut self) -> Result<Vec<ClassNode>, AnalysisError> {
        let files = self.context()?.source_map().files;
        let mut nodes = Vec::new();
        for source_file in files {
            let Some(module) = self.file_to_module.get(&source_file.id).copied() else {
                continue;
            };
            let analysis = self.context_mut()?.analysis_for_module(module).into_value();
            let parsed = self.context_mut()?.parse_module(module).into_value().parsed;
            for class in analysis.classes {
                let Some(declaration) = parsed.suite().iter().find_map(|statement| {
                    let Stmt::ClassDef(declaration) = statement else {
                        return None;
                    };
                    (declaration.name.as_str() == class.name).then_some(declaration)
                }) else {
                    continue;
                };
                nodes.push(class_node(source_file.id, &class, declaration));
            }
        }
        Ok(nodes)
    }
}

fn class_node(
    file: FileId,
    class: &ClassHierarchyView,
    declaration: &sifr_python_ast::StmtClassDef,
) -> ClassNode {
    let parent_range = class.parent_name.as_ref().and_then(|parent| {
        declaration.arguments.as_ref().and_then(|arguments| {
            arguments
                .args
                .iter()
                .find(|expr| base_name(expr) == Some(parent.as_str()))
                .map(base_range)
        })
    });
    ClassNode {
        item: TypeHierarchyItem {
            id: TypeHierarchyItemId(format!("{}:{}", file.as_u32(), class.name)),
            name: class.name.clone(),
            kind: "class".to_string(),
            location: Location {
                file,
                range: Some(declaration.name.range()),
            },
        },
        identity: class.identity.clone(),
        parent_identity: class.parent_identity.clone(),
        parent_range,
    }
}

fn base_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Subscript(subscript) => base_name(&subscript.value),
        Expr::Name(name) => Some(name.id.as_str()),
        Expr::Attribute(attribute) => Some(attribute.attr.as_str()),
        _ => None,
    }
}

fn base_range(expr: &Expr) -> TextRange {
    match expr {
        Expr::Subscript(subscript) => base_range(&subscript.value),
        Expr::Attribute(attribute) => attribute.attr.range(),
        _ => expr.range(),
    }
}

fn has_hierarchy_edge(node: &ClassNode, nodes: &[ClassNode]) -> bool {
    node.parent_identity
        .as_ref()
        .is_some_and(|identity| nodes.iter().any(|other| other.identity == *identity))
        || nodes
            .iter()
            .any(|other| other.parent_identity.as_deref() == Some(node.identity.as_str()))
}

fn contains(range: TextRange, offset: TextSize) -> bool {
    range.start() <= offset && offset < range.end()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_frontend::{
        DocumentVersion, FrontendInput, FrontendMode, ProjectRoot, SourcePath, SourceText,
    };

    fn compiler() -> sifr_compiler_services::CompilerContext {
        sifr_compiler_services::CompilerContext::for_test_tokens(
            crate::compiled_input_tokens(),
            "sifr_analysis-type-hierarchy-tests",
        )
    }

    fn position(line: u32, character: u32) -> TextPosition {
        TextPosition { line, character }
    }

    #[test]
    fn lowercase_types_have_edges_and_uppercase_values_do_not() {
        let initial = "class lower:\n    pass\nclass Child(lower):\n    pass\nUPPER: int = 1\n";
        let mut host = AnalysisHost::open_single_file(
            &compiler(),
            FrontendInput {
                path: SourcePath::new("main.sifr"),
                source: SourceText::new(initial),
                mode: FrontendMode::SingleFile,
            },
        )
        .expect("hierarchy source should load");
        let file = host.files()[0];
        let base = host
            .prepare_type_hierarchy(file, &position(0, 7))
            .expect("prepare base")
            .into_value()
            .expect("lowercase class is a type");
        let child = host
            .prepare_type_hierarchy(file, &position(2, 8))
            .expect("prepare child")
            .into_value()
            .expect("child is a type");
        assert_eq!(
            host.type_hierarchy_supertypes(child.id.clone())
                .expect("supertypes")
                .into_value(),
            vec![base.clone()]
        );
        assert_eq!(
            host.type_hierarchy_subtypes(base.id.clone())
                .expect("subtypes")
                .into_value(),
            vec![child]
        );
        assert!(
            host.prepare_type_hierarchy(file, &position(4, 2))
                .expect("prepare value")
                .into_value()
                .is_none()
        );

        host.update_document(
            file,
            DocumentVersion::new(2),
            SourceText::new("class lower:\n    pass\nclass Child:\n    pass\nUPPER: int = 1\n"),
        )
        .expect("edit should load");
        assert!(
            host.type_hierarchy_subtypes(base.id.clone())
                .expect("updated subtypes")
                .into_value()
                .is_empty()
        );
        assert!(
            host.prepare_type_hierarchy(file, &position(0, 7))
                .expect("updated prepare")
                .into_value()
                .is_none()
        );
    }

    #[test]
    fn imported_base_resolves_to_its_declaration() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("sifr_hierarchy_{}_{nonce}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("project directory");
        std::fs::write(dir.join("base.sifr"), "class lower:\n    pass\n").expect("base source");
        std::fs::write(
            dir.join("main.sifr"),
            "from base import lower as Imported\nclass Child(Imported):\n    pass\n",
        )
        .expect("main source");
        let mut host = AnalysisHost::open_project(
            &compiler(),
            &ProjectRoot {
                root: SourcePath::new(&dir),
                entrypoint: SourcePath::new(dir.join("main.sifr")),
            },
        )
        .expect("project should load");
        let main = host
            .document_file_for_path(&dir.join("main.sifr"))
            .expect("main file");
        let base_file = host
            .document_file_for_path(&dir.join("base.sifr"))
            .expect("base file");
        let child = host
            .prepare_type_hierarchy(main, &position(1, 7))
            .expect("prepare child")
            .into_value()
            .expect("child type");
        let parents = host
            .type_hierarchy_supertypes(child.id.clone())
            .expect("parents")
            .into_value();
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].name, "lower");
        assert_eq!(parents[0].location.file, base_file);
        assert_eq!(
            host.prepare_type_hierarchy(main, &position(1, 13))
                .expect("prepare imported base reference")
                .into_value(),
            Some(parents[0].clone())
        );
        assert_eq!(
            host.type_hierarchy_subtypes(parents[0].id.clone())
                .expect("subtypes")
                .into_value(),
            vec![child.clone()]
        );
        host.update_document(
            base_file,
            DocumentVersion::new(2),
            SourceText::new("class other:\n    pass\n"),
        )
        .expect("base edit should load");
        assert!(
            host.type_hierarchy_supertypes(child.id)
                .expect("stale imported base should disappear")
                .into_value()
                .is_empty()
        );
        std::fs::remove_dir_all(dir).expect("project cleanup");
    }
}
