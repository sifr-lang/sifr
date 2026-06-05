use crate::RustEmitter;
use sifr_ir::HirPattern;

impl RustEmitter {
    pub(crate) fn substitute_class_captures_in_guard(
        guard_code: &str,
        pattern: &HirPattern,
        is_non_option_union: bool,
    ) -> String {
        if let HirPattern::Class { fields, .. } = pattern {
            let prefix = if is_non_option_union {
                "__inner"
            } else {
                "__matched"
            };
            let mut result = guard_code.to_string();
            for (fname, fpat) in fields {
                if let HirPattern::Capture { name, .. } = fpat {
                    let replacement = format!("{prefix}.{fname}");
                    result = Self::replace_identifier(&result, name, &replacement);
                }
            }
            result
        } else {
            guard_code.to_string()
        }
    }

    pub(crate) fn replace_identifier(code: &str, ident: &str, replacement: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = code.chars().collect();
        let ident_chars: Vec<char> = ident.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if i + ident_chars.len() <= chars.len()
                && &chars[i..i + ident_chars.len()] == ident_chars.as_slice()
            {
                let before_ok = i == 0 || !chars[i - 1].is_alphanumeric() && chars[i - 1] != '_';
                let after_ok = i + ident_chars.len() >= chars.len()
                    || !chars[i + ident_chars.len()].is_alphanumeric()
                        && chars[i + ident_chars.len()] != '_';
                if before_ok && after_ok {
                    result.push_str(replacement);
                    i += ident_chars.len();
                    continue;
                }
            }
            result.push(chars[i]);
            i += 1;
        }
        result
    }
}
