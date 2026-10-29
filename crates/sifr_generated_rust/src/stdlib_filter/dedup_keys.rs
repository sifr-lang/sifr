use quote::ToTokens;
use syn::ItemImpl;

pub(super) fn dedup_impl_key(item_impl: &ItemImpl) -> String {
    let mut header = item_impl.clone();
    header.items.clear();
    let header = header.to_token_stream().to_string();
    if item_impl.trait_.is_some() {
        return header;
    }

    let member_signatures = item_impl
        .items
        .iter()
        .map(|item| match item {
            syn::ImplItem::Const(item) => {
                format!("const {} : {}", item.ident, item.ty.to_token_stream())
            }
            syn::ImplItem::Fn(item) => item.sig.to_token_stream().to_string(),
            syn::ImplItem::Type(item) => format!("type {}", item.ident),
            syn::ImplItem::Macro(item) => item.mac.path.to_token_stream().to_string(),
            syn::ImplItem::Verbatim(tokens) => format!("verbatim {tokens}"),
            _ => "unknown".to_string(),
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{header} [{member_signatures}]")
}
