//! Expression lowering scaffolds for the IR lowering.

include!("lower_expr/leaves_and_plain_calls.rs");
include!("lower_expr/iterators_and_callables.rs");
include!("lower_expr/collections_and_comprehensions.rs");

#[cfg(test)]
mod tests {
    include!("lower_expr/leaves_and_compound_tests.rs");
    include!("lower_expr/option_compare_tests.rs");
    include!("lower_expr/comprehension_and_misc_tests.rs");
}
