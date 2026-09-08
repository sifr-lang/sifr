//! Syntax reuse belongs to one bootstrap emission session, never the global cache.
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Range;

use syn::parse::{ParseStream, Parser};
use syn::spanned::Spanned;

/// Retains successful support syntax identities while emitting a stdlib inventory.
/// Dropping the session releases all identities, including on bootstrap failure.
#[derive(Default)]
pub struct StdlibSyntaxSession {
    validated_support: RefCell<HashSet<String>>,
    #[cfg(test)]
    reused: std::cell::Cell<usize>,
}

impl StdlibSyntaxSession {
    pub(crate) fn validate(&self, source: &str, support: Range<usize>) -> syn::Result<()> {
        // syn owns BOM/shebang interpretation (including comments before '[').
        // These sources are not renderer support-cache candidates.
        if source.starts_with('\u{feff}') || source.starts_with("#!") {
            return syn::parse_file(source).map(|_| ());
        }
        let support_source = &source[support.clone()];
        let known =
            !support_source.is_empty() && self.validated_support.borrow().contains(support_source);
        let mut first = None;
        let mut last = None;
        let mut closed_end = false;
        let mut reused = false;
        let parser = |input: ParseStream<'_>| {
            // This is syn::File's grammar: inner attributes, then complete Items.
            // parse_str tokenizes the ENTIRE assembly before entering this parser,
            // so delimiters, comments, literals and fragment joins remain checked.
            input.call(syn::Attribute::parse_inner)?;
            while !input.is_empty() {
                if known && consume_validated_support(input, &support)? {
                    reused = true;
                    continue;
                }
                let item: syn::Item = input.parse()?;
                let span = item.span().byte_range();
                if span.start >= support.start && span.end <= support.end {
                    first.get_or_insert(span.start);
                    last = Some(span.end);
                    // Only unconditionally closed item forms may seal a cached
                    // suffix. In particular, macros may consume a following ';'.
                    closed_end = matches!(
                        item,
                        syn::Item::Fn(_)
                            | syn::Item::Impl(_)
                            | syn::Item::Struct(_)
                            | syn::Item::Enum(_)
                    );
                }
            }
            Ok(())
        };
        parser.parse_str(source)?;
        // Publication follows full-file success. Partial or invalid assemblies
        // never establish a reusable support identity.
        if first == Some(support.start) && last == Some(support.end) && closed_end {
            self.validated_support
                .borrow_mut()
                .insert(support_source.to_owned());
        }
        #[cfg(test)]
        if reused {
            self.reused.set(self.reused.get() + 1);
        }
        #[cfg(not(test))]
        let _ = reused;
        Ok(())
    }
}

fn consume_validated_support(input: ParseStream<'_>, support: &Range<usize>) -> syn::Result<bool> {
    input.step(|cursor| {
        let begin = *cursor;
        if begin.span().byte_range().start != support.start {
            return Ok((false, begin));
        }
        let mut current = begin;
        while let Some((token, next)) = current.token_tree() {
            let span = token.span().byte_range();
            // A group/literal spanning a join has different lexical ownership,
            // even if the support substring is identical. syn must parse it.
            if span.start < support.start || span.end > support.end {
                return Ok((false, begin));
            }
            if span.end == support.end {
                return Ok((true, next));
            }
            current = next;
        }
        Ok((false, begin))
    })
}

#[cfg(test)]
#[path = "inline_syntax_tests.rs"]
mod inline_syntax_tests;
