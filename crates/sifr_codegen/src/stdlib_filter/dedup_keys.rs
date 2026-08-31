use syn::{ItemImpl, Type};

pub(super) fn dedup_impl_key(item_impl: &ItemImpl) -> String {
    let self_ty = dedup_type_key(item_impl.self_ty.as_ref());
    if let Some((trait_path, _)) = &item_impl.trait_ {
        let defaultness = if item_impl.modifiers.defaultness.is_some() {
            "default "
        } else {
            ""
        };
        let polarity = if item_impl.modifiers.polarity.is_some() {
            "!"
        } else {
            ""
        };
        format!(
            "{defaultness}impl {polarity}{} for {}",
            dedup_path_key(trait_path),
            self_ty
        )
    } else {
        let item_names = item_impl
            .items
            .iter()
            .map(|item| match item {
                syn::ImplItem::Const(item) => format!("const {}", item.ident),
                syn::ImplItem::Fn(item) => format!("fn {}", item.sig.ident),
                syn::ImplItem::Type(item) => format!("type {}", item.ident),
                syn::ImplItem::Macro(item) => format!("macro {}", dedup_path_key(&item.mac.path)),
                syn::ImplItem::Verbatim(tokens) => format!("verbatim {tokens}"),
                _ => "unknown".to_string(),
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("impl {self_ty} [{item_names}]")
    }
}

fn dedup_path_key(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

fn dedup_type_key(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => dedup_path_key(&type_path.path),
        Type::Reference(reference) => dedup_type_key(reference.elem.as_ref()),
        Type::Paren(paren) => dedup_type_key(paren.elem.as_ref()),
        Type::Group(group) => dedup_type_key(group.elem.as_ref()),
        Type::Slice(slice) => format!("[{}]", dedup_type_key(slice.elem.as_ref())),
        Type::Array(array) => format!("[{}]", dedup_type_key(array.elem.as_ref())),
        Type::Tuple(tuple) => {
            let elems = tuple
                .elems
                .iter()
                .map(dedup_type_key)
                .collect::<Vec<String>>()
                .join(",");
            format!("({elems})")
        }
        _ => "__unknown_type__".to_string(),
    }
}
