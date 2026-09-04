use std::collections::HashMap;
use std::collections::HashSet;

use syn::visit_mut::{self, VisitMut};

pub(super) fn collect_borrowed_string_params(file: &syn::File) -> HashMap<String, Vec<bool>> {
    let mut collector = BorrowedStringParamCollector::default();
    syn::visit::Visit::visit_file(&mut collector, file);
    collector.params
}

pub(super) fn collect_owned_string_returns(file: &syn::File) -> HashSet<String> {
    let mut returns = HashSet::new();
    let mut collector = OwnedStringReturnCollector {
        returns: &mut returns,
    };
    syn::visit::Visit::visit_file(&mut collector, file);
    returns
}

struct OwnedStringReturnCollector<'returns> {
    returns: &'returns mut HashSet<String>,
}

impl syn::visit::Visit<'_> for OwnedStringReturnCollector<'_> {
    fn visit_signature(&mut self, signature: &syn::Signature) {
        if matches!(&signature.output, syn::ReturnType::Type(_, ty)
            if matches!(ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("String")))
        {
            let argument_count = signature
                .inputs
                .iter()
                .filter(|argument| matches!(argument, syn::FnArg::Typed(_)))
                .count();
            self.returns
                .insert(signature_key(&signature.ident.to_string(), argument_count));
        }
        syn::visit::visit_signature(self, signature);
    }
}

pub(super) fn remove_returned_string_conversion(
    expression: &mut syn::Expr,
    returns: &HashSet<String>,
) {
    let syn::Expr::MethodCall(conversion) = expression else {
        return;
    };
    if !matches!(
        conversion.method.to_string().as_str(),
        "clone" | "to_owned" | "to_string"
    ) || !conversion.args.is_empty()
    {
        return;
    }
    let (name, argument_count) = match conversion.receiver.as_ref() {
        syn::Expr::Call(call) => {
            let syn::Expr::Path(path) = call.func.as_ref() else {
                return;
            };
            let Some(name) = path.path.segments.last() else {
                return;
            };
            (name.ident.to_string(), call.args.len())
        }
        syn::Expr::MethodCall(call) => (call.method.to_string(), call.args.len()),
        _ => return,
    };
    if returns.contains(&signature_key(&name, argument_count)) {
        *expression = conversion.receiver.as_ref().clone();
    }
}

pub(crate) fn collect_project_borrowed_string_params(
    files: &[syn::File],
) -> HashMap<String, Vec<bool>> {
    let mut merged = HashMap::new();
    for file in files {
        for (name, params) in collect_borrowed_string_params(file) {
            merged
                .entry(name)
                .and_modify(|known: &mut Vec<bool>| {
                    if *known != params {
                        known.clear();
                    }
                })
                .or_insert(params);
        }
    }
    merged
}

pub(crate) fn rewrite_project_borrowed_string_literals(
    file: &mut syn::File,
    signatures: &HashMap<String, Vec<bool>>,
) {
    ProjectBorrowedStringLiteralRewriter { signatures }.visit_file_mut(file);
}

struct ProjectBorrowedStringLiteralRewriter<'signatures> {
    signatures: &'signatures HashMap<String, Vec<bool>>,
}

impl VisitMut for ProjectBorrowedStringLiteralRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        rewrite_borrowed_string_literal_arguments(expression, self.signatures);
    }
}

#[derive(Default)]
struct BorrowedStringParamCollector {
    params: HashMap<String, Vec<bool>>,
}

impl<'ast> syn::visit::Visit<'ast> for BorrowedStringParamCollector {
    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        let params = signature
            .inputs
            .iter()
            .filter_map(|argument| match argument {
                syn::FnArg::Typed(parameter) => Some(
                    matches!(parameter.ty.as_ref(), syn::Type::Reference(reference)
                    if matches!(reference.elem.as_ref(), syn::Type::Path(path)
                        if path.path.is_ident("str"))),
                ),
                syn::FnArg::Receiver(_) => None,
            })
            .collect::<Vec<_>>();
        self.params
            .entry(signature_key(&signature.ident.to_string(), params.len()))
            .and_modify(|known| {
                if *known != params {
                    known.clear();
                }
            })
            .or_insert(params);
        syn::visit::visit_signature(self, signature);
    }
}

fn signature_key(name: &str, argument_count: usize) -> String {
    format!("{name}#{argument_count}")
}

pub(super) fn rewrite_borrowed_string_literal_arguments(
    expression: &mut syn::Expr,
    signatures: &HashMap<String, Vec<bool>>,
) {
    let (name, arguments, runtime_path) = match expression {
        syn::Expr::Call(call) => {
            let syn::Expr::Path(path) = call.func.as_ref() else {
                return;
            };
            let Some(name) = path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string())
            else {
                return;
            };
            let runtime_path = path
                .path
                .segments
                .iter()
                .any(|segment| segment.ident == "sifr_runtime");
            (name, &mut call.args, runtime_path)
        }
        syn::Expr::MethodCall(call) => (call.method.to_string(), &mut call.args, false),
        _ => return,
    };
    let key = signature_key(&name, arguments.len());
    let known_params = signatures.get(&key).filter(|params| !params.is_empty());
    if known_params.is_none() && !runtime_path {
        return;
    }
    for (index, argument) in arguments.iter_mut().enumerate() {
        let borrowed_string = known_params
            .and_then(|params| params.get(index))
            .copied()
            .unwrap_or(runtime_path);
        if !borrowed_string {
            continue;
        }
        let syn::Expr::Reference(reference) = argument else {
            continue;
        };
        let syn::Expr::MethodCall(conversion) = reference.expr.as_mut() else {
            continue;
        };
        if matches!(
            conversion.method.to_string().as_str(),
            "to_owned" | "to_string"
        ) && conversion.args.is_empty()
            && matches!(conversion.receiver.as_ref(), syn::Expr::Lit(literal)
                if matches!(literal.lit, syn::Lit::Str(_)))
        {
            *argument = conversion.receiver.as_ref().clone();
        } else if conversion.method == "clone"
            && conversion.args.is_empty()
            && matches!(conversion.receiver.as_ref(), syn::Expr::Path(_))
        {
            conversion.method = syn::Ident::new("to_string", conversion.method.span());
        }
    }
}
