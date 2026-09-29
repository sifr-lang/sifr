//! Seeded ownership transfer properties through the canonical frontend product.
//! Set `SIFR_H01G_CASE` to a failing case index to replay its source.
use crate::{
    FrontendDiagnosticStyle, FrontendProductInput, compile_frontend_product, parse_source,
};
use sifr_lowering::{ExternalDefs, LoweringOptions};
use std::collections::HashMap;
use std::fmt::Write;

const SEED: u64 = 0x48_30_31_67_4f_57_4e_52;
const CASES: usize = 512;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn pick(&mut self, limit: u64) -> u64 {
        self.next() % limit
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Available,
    Moved,
}

struct Program {
    source: String,
    expected_error: Option<&'static str>,
    invariant: &'static str,
}

fn generated_program(rng: &mut Rng, case: usize) -> Program {
    let is_list = rng.pick(2) == 0;
    let ty = if is_list { "list[int]" } else { "str" };
    let literal = if is_list {
        format!("[{}, {}]", rng.pick(100), rng.pick(100))
    } else {
        format!("\"word_{}_{}\"", rng.pick(100), rng.pick(100))
    };
    let value = format!("value_{case}");
    let consume = format!("consume_{case}");
    let borrow = format!("borrow_{case}");
    let mut source = format!(
        "def {consume}(own item: {ty}) -> int:\n    return len(item)\n\ndef {borrow}(item: {ty}) -> int:\n    return len(item)\n\ndef main() -> None:\n    {value}: {ty} = {literal}\n"
    );
    for _ in 0..=rng.pick(4) {
        if rng.pick(2) == 0 {
            writeln!(source, "    print({borrow}({value}))")
                .expect("String formatting cannot fail");
        } else {
            writeln!(source, "    print(len({value}))").expect("String formatting cannot fail");
        }
    }
    let family = case % 8;
    let (expected_error, invariant) = match family {
        0 => {
            writeln!(source, "    print({borrow}({value}))")
                .expect("String formatting cannot fail");
            (None, "shared reads preserve ownership")
        }
        1..=3 => {
            let mut state = State::Available;
            for step in 0..=rng.pick(4) {
                if state == State::Moved {
                    writeln!(source, "    {value} = {literal}")
                        .expect("String formatting cannot fail");
                    state = State::Available;
                } else {
                    match rng.pick(4) {
                        0 => {
                            writeln!(source, "    print({borrow}({value}))")
                                .expect("String formatting cannot fail");
                        }
                        1 => {
                            writeln!(source, "    print(len({value}))")
                                .expect("String formatting cannot fail");
                        }
                        2 => {
                            writeln!(source, "    interim_{case}_{step}: {ty} = {value}")
                                .expect("String formatting cannot fail");
                            state = State::Moved;
                        }
                        _ => {
                            writeln!(source, "    print({consume}({value}))")
                                .expect("String formatting cannot fail");
                            state = State::Moved;
                        }
                    }
                }
            }
            if state == State::Moved {
                writeln!(source, "    {value} = {literal}").expect("String formatting cannot fail");
            }
            if family == 1 {
                writeln!(source, "    moved_{case}: {ty} = {value}")
                    .expect("String formatting cannot fail");
            } else {
                writeln!(source, "    print({consume}({value}))")
                    .expect("String formatting cannot fail");
            }
            state = State::Moved;
            if family == 3 {
                writeln!(source, "    {value} = {literal}").expect("String formatting cannot fail");
                state = State::Available;
            }
            writeln!(source, "    print(len({value}))").expect("String formatting cannot fail");
            (
                (state == State::Moved).then_some("SIFR-OWN-0001"),
                "a move invalidates the source until rebinding",
            )
        }
        4 => {
            write!(source,
                "    if len({value}) > {}:\n        print({consume}({value}))\n    else:\n        print({borrow}({value}))\n    print(len({value}))\n",
                rng.pick(5)
            ).expect("String formatting cannot fail");
            (
                Some("SIFR-OWN-0001"),
                "a possible move remains moved at a branch join",
            )
        }
        5 => {
            write!(source,
                "    if len({value}) > {}:\n        print({borrow}({value}))\n    else:\n        print(len({value}))\n    print(len({value}))\n",
                rng.pick(5)
            ).expect("String formatting cannot fail");
            (None, "borrows on both branches preserve ownership")
        }
        6 | 7 => {
            source = format!(
                "def overlap(mut left: list[int], right: list[int]) -> None:\n    left.append(len(right))\n\ndef main() -> None:\n    first: list[int] = [{}, {}]\n    second: list[int] = [{}]\n",
                rng.pick(100),
                rng.pick(100),
                rng.pick(100)
            );
            source.push_str(if family == 6 {
                "    overlap(first, first)\n"
            } else {
                "    overlap(first, second)\n"
            });
            source.push_str("    print(len(first))\n");
            (
                (family == 6).then_some("SIFR-OWN-0002"),
                "mutable and shared borrows cannot alias in one call",
            )
        }
        _ => unreachable!(),
    };
    Program {
        source,
        expected_error,
        invariant,
    }
}

fn compile(source: &str) -> Result<(), Vec<sifr_diagnostics::RenderedDiagnostic>> {
    let suite = parse_source(source, Some("property.sifr"))?;
    let inputs = HashMap::from([(
        "main".to_string(),
        FrontendProductInput {
            suite: &suite,
            source,
            display_path: "property.sifr",
            source_backed: true,
        },
    )]);
    compile_frontend_product(
        &inputs,
        ExternalDefs::default(),
        FrontendDiagnosticStyle::Bare,
        &LoweringOptions::default(),
    )
    .map(|_| ())
}

#[test]
fn move_borrow_branch_invariants() {
    let selected = std::env::var("SIFR_H01G_CASE")
        .ok()
        .map(|value| value.parse::<usize>().expect("case index is an integer"));
    let mut rng = Rng(SEED);
    let mut executed = 0;
    for case in 0..CASES {
        let program = generated_program(&mut rng, case);
        if selected.is_some_and(|index| index != case) {
            continue;
        }
        executed += 1;
        let actual = compile(&program.source);
        match program.expected_error {
            None => assert!(
                actual.is_ok(),
                "seed={SEED:#x} case={case} invariant={} expected acceptance, got {actual:?}\n{}",
                program.invariant,
                program.source
            ),
            Some(code) => assert!(
                actual.as_ref().is_err_and(|diagnostics| {
                    diagnostics.len() == 1 && diagnostics[0].code == code
                }),
                "seed={SEED:#x} case={case} invariant={} expected {code}, got {actual:?}\n{}",
                program.invariant,
                program.source
            ),
        }
    }
    assert_eq!(
        executed,
        selected.map_or(CASES, |_| 1),
        "case index out of range"
    );
}

#[test]
fn existing_ownership_fixture_regressions() {
    let accepted = [
        (
            "borrowed_parameters",
            include_str!("../../sifr/tests/e2e/pass/borrowed_parameters.sifr"),
        ),
        (
            "ownership_print_reuse",
            include_str!("../../sifr/tests/e2e/pass/ownership_print_reuse.sifr"),
        ),
        (
            "mut_borrowed_parameter_field_mutation",
            include_str!("../../sifr/tests/e2e/pass/mut_borrowed_parameter_field_mutation.sifr"),
        ),
    ];
    for (name, source) in accepted {
        assert!(compile(source).is_ok(), "{name} should pass");
    }
    let rejected = [
        (
            "use_after_move",
            include_str!("../../sifr/tests/e2e/fail/use_after_move.sifr"),
            "SIFR-OWN-0001",
        ),
        (
            "use_after_move_assign",
            include_str!("../../sifr/tests/e2e/fail/use_after_move_assign.sifr"),
            "SIFR-OWN-0001",
        ),
        (
            "double_mut_borrow",
            include_str!("../../sifr/tests/e2e/fail/double_mut_borrow.sifr"),
            "SIFR-OWN-0002",
        ),
        (
            "mut_borrow_after_immutable_borrow",
            include_str!("../../sifr/tests/e2e/fail/mut_borrow_after_immutable_borrow.sifr"),
            "SIFR-OWN-0002",
        ),
        (
            "immutable_parameter_indexed_list_receiver",
            include_str!(
                "../../sifr/tests/e2e/fail/immutable_parameter_indexed_list_receiver.sifr"
            ),
            "SIFR-OWN-0005",
        ),
    ];
    for (name, source, code) in rejected {
        let diagnostics = compile(source).expect_err(name);
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.code == code),
            "{name} should report {code}: {diagnostics:?}"
        );
    }
}
