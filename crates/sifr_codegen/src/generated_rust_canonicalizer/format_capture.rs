use std::collections::HashSet;
use std::ops::Range;

use quote::ToTokens;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Capture {
    name: String,
    range: Range<usize>,
}

pub(crate) fn names(rust_macro: &syn::Macro) -> HashSet<String> {
    format_string(rust_macro)
        .map(|format| {
            captures(&format)
                .into_iter()
                .map(|capture| capture.name)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn rename(rust_macro: &mut syn::Macro, from: &str, to: &str) -> bool {
    let Some(format_index) = format_argument_index(rust_macro) else {
        return false;
    };
    let Ok(mut arguments) = rust_macro.parse_body_with(
        syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
    ) else {
        return false;
    };
    let Some(syn::Expr::Lit(format_expression)) = arguments.iter_mut().nth(format_index) else {
        return false;
    };
    let syn::Lit::Str(format_literal) = &mut format_expression.lit else {
        return false;
    };
    let mut format = format_literal.value();
    let replacements = captures(&format)
        .into_iter()
        .filter(|capture| capture.name == from)
        .map(|capture| capture.range)
        .collect::<Vec<_>>();
    if replacements.is_empty() {
        return false;
    }
    for range in replacements.into_iter().rev() {
        format.replace_range(range, to);
    }
    *format_literal = syn::LitStr::new(&format, format_literal.span());
    rust_macro.tokens = arguments.into_token_stream();
    true
}

fn format_string(rust_macro: &syn::Macro) -> Option<String> {
    let format_index = format_argument_index(rust_macro)?;
    let arguments = rust_macro
        .parse_body_with(syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
        .ok()?;
    let syn::Expr::Lit(format_expression) = arguments.iter().nth(format_index)? else {
        return None;
    };
    let syn::Lit::Str(format_literal) = &format_expression.lit else {
        return None;
    };
    Some(format_literal.value())
}

fn format_argument_index(rust_macro: &syn::Macro) -> Option<usize> {
    let name = rust_macro.path.segments.last()?.ident.to_string();
    match name.as_str() {
        "format" | "format_args" | "format_args_nl" | "print" | "println" | "eprint"
        | "eprintln" | "panic" | "unreachable" | "todo" | "unimplemented" => Some(0),
        "assert" | "debug_assert" | "write" | "writeln" => Some(1),
        "assert_eq" | "assert_ne" | "debug_assert_eq" | "debug_assert_ne" => Some(2),
        _ => None,
    }
}

fn captures(format: &str) -> Vec<Capture> {
    let bytes = format.as_bytes();
    let mut captures = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let Some(relative_start) = format[offset..].find('{') else {
            break;
        };
        let start = offset + relative_start;
        if bytes.get(start + 1) == Some(&b'{') {
            offset = start + 2;
            continue;
        }
        let Some(relative_end) = format[start + 1..].find('}') else {
            break;
        };
        let end = start + 1 + relative_end;
        collect_field_captures(format, start + 1, end, &mut captures);
        offset = end + 1;
    }
    captures
}

fn collect_field_captures(format: &str, start: usize, end: usize, captures: &mut Vec<Capture>) {
    let field = &format[start..end];
    let separator = field.find(':');
    let argument_end = separator.map_or(end, |relative| start + relative);
    push_identifier_capture(format, start..argument_end, captures);

    let Some(separator) = separator else {
        return;
    };
    let spec_start = start + separator + 1;
    let mut cursor = spec_start;
    while cursor < end {
        let Some(identifier_end) = identifier_end(format, cursor, end) else {
            cursor += format[cursor..end].chars().next().map_or(1, char::len_utf8);
            continue;
        };
        if format.as_bytes().get(identifier_end) == Some(&b'$') {
            push_identifier_capture(format, cursor..identifier_end, captures);
        }
        cursor = identifier_end.max(cursor + 1);
    }
}

fn push_identifier_capture(format: &str, range: Range<usize>, captures: &mut Vec<Capture>) {
    let name = &format[range.clone()];
    if is_identifier(name) {
        captures.push(Capture {
            name: name.to_string(),
            range,
        });
    }
}

fn identifier_end(text: &str, start: usize, end: usize) -> Option<usize> {
    let mut cursor = start;
    if text[cursor..end].starts_with("r#") {
        cursor += 2;
    }
    let first = text[cursor..end].chars().next()?;
    if !is_identifier_start(first) {
        return None;
    }
    cursor += first.len_utf8();
    while cursor < end {
        let Some(character) = text[cursor..end].chars().next() else {
            break;
        };
        if !is_identifier_continue(character) {
            break;
        }
        cursor += character.len_utf8();
    }
    Some(cursor)
}

fn is_identifier(name: &str) -> bool {
    identifier_end(name, 0, name.len()) == Some(name.len())
        && (name == "self" || syn::parse_str::<syn::Ident>(name).is_ok())
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || unicode_ident::is_xid_start(character)
}

fn is_identifier_continue(character: char) -> bool {
    character == '_' || unicode_ident::is_xid_continue(character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_value_width_and_precision_captures() {
        let rust_macro: syn::Macro = syn::parse_quote!(println!(
            "{{escaped}} {value:>width$.precision$} {:fallback_width$}"
        ));
        assert_eq!(
            names(&rust_macro),
            HashSet::from([
                "fallback_width".to_string(),
                "precision".to_string(),
                "value".to_string(),
                "width".to_string(),
            ])
        );

        let format_args: syn::Macro = syn::parse_quote!(format_args!("{value:width$}"));
        assert_eq!(
            names(&format_args),
            HashSet::from(["value".to_string(), "width".to_string()])
        );
    }

    #[test]
    fn renames_every_capture_role() {
        let mut rust_macro: syn::Macro = syn::parse_quote!(println!("{value:value$.value$}"));
        assert!(rename(&mut rust_macro, "value", "renamed"));
        assert!(
            rust_macro
                .tokens
                .to_string()
                .contains("{renamed:renamed$.renamed$}")
        );
    }

    #[test]
    fn item12_routes_implicit_capture_macro_families_to_their_format_argument() {
        for rust_macro in [
            syn::parse_quote!(panic!("{value}")),
            syn::parse_quote!(unreachable!("{value}")),
            syn::parse_quote!(todo!("{value}")),
            syn::parse_quote!(unimplemented!("{value}")),
            syn::parse_quote!(debug_assert!(false, "{value}")),
            syn::parse_quote!(debug_assert_eq!(1, 2, "{value}")),
        ] {
            assert_eq!(names(&rust_macro), HashSet::from(["value".to_string()]));
        }
    }
}
