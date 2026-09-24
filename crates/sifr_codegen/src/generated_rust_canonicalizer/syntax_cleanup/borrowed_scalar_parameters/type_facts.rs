#[derive(Default)]
struct SifrIntBindingCollector {
    names: HashSet<String>,
}

impl Visit<'_> for SifrIntBindingCollector {
    fn visit_local(&mut self, local: &syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat
            && type_is_sifr_int(&typed.ty)
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            self.names.insert(name);
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

fn type_is_sifr_int(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(reference) => type_is_sifr_int(&reference.elem),
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "SifrInt"),
        _ => false,
    }
}

fn owned_scalar_type(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "SifrInt" || owned_sifr_int_option_type(ty))
}

fn borrowed_scalar_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Reference(reference) if owned_scalar_type(&reference.elem))
}

fn owned_sifr_int_option_type(ty: &syn::Type) -> bool {
    owned_sifr_int_option_inner(ty).is_some()
}

fn owned_sifr_int_option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    if segment.ident != "Option" {
        return None;
    }
    arguments.args.iter().find_map(|argument| {
        let syn::GenericArgument::Type(inner) = argument else {
            return None;
        };
        matches!(inner, syn::Type::Path(path)
            if path.path.segments.last().is_some_and(|segment| segment.ident == "SifrInt"))
        .then_some(inner)
    })
}

fn borrowed_sifr_int_option_type(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    segment.ident == "Option"
        && matches!(arguments.args.first(), Some(syn::GenericArgument::Type(syn::Type::Reference(reference)))
            if type_is_sifr_int(&reference.elem))
}

fn expression_mentions_name(expression: &syn::Expr, name: &str) -> bool {
    let mut finder = NameFinder::new(name);
    finder.visit_expr(expression);
    finder.found
}

struct NameFinder<'name> {
    name: &'name str,
    found: bool,
}

impl<'name> NameFinder<'name> {
    fn new(name: &'name str) -> Self {
        Self { name, found: false }
    }
}

impl Visit<'_> for NameFinder<'_> {
    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.found = true;
        }
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}
