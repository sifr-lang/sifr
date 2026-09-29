//! Seeded semantic properties. A failing assertion prints the seed and case
//! index so the exact generated input can be replayed with the named test.

use crate::{LiteralValue, NarrowingCondition, Type, make_union, narrow_type};

const NORMALIZATION_SEED: u64 = 0x48_30_31_66_4e_4f_52_4d;
const NARROWING_SEED: u64 = 0x48_30_31_66_4e_41_52_52;
const CASES: usize = 2_048;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn index(&mut self, len: usize) -> usize {
        let bound = u64::try_from(len).expect("test generator bound fits u64");
        usize::try_from(self.next() % bound).expect("bounded test index fits usize")
    }
}

fn atom(index: usize) -> Type {
    match index % 10 {
        0 => Type::Int,
        1 => Type::Str,
        2 => Type::Bool,
        3 => Type::Bytes,
        4 => Type::None,
        5 => Type::LiteralInt(0),
        6 => Type::LiteralInt(7),
        7 => Type::LiteralStr(String::new()),
        8 => Type::LiteralStr("x".to_string()),
        _ => Type::LiteralBool(false),
    }
}

#[test]
fn normalization_idempotent() {
    let mut rng = Rng(NORMALIZATION_SEED);
    for case in 0..CASES {
        let count = 1 + rng.index(8);
        let mut members = Vec::with_capacity(count + 2);
        for _ in 0..count {
            let first = atom(rng.index(10));
            let member = match rng.index(4) {
                0 => Type::Union(vec![first.clone(), first]),
                1 => Type::Union(vec![first, Type::None]),
                _ => first,
            };
            members.push(member);
        }
        if case % 3 == 0 {
            members.push(Type::Union(vec![Type::Str, Type::None]));
        }
        let normalized = make_union(members.clone());
        let mut permuted = members.clone();
        permuted.reverse();
        permuted.push(members[rng.index(members.len())].clone());
        assert_eq!(
            normalized,
            make_union(vec![normalized.clone()]),
            "seed={NORMALIZATION_SEED:#x} case={case} members={members:?}"
        );
        assert_eq!(
            normalized,
            make_union(permuted),
            "seed={NORMALIZATION_SEED:#x} case={case} members={members:?}"
        );
        let raw = Type::Union(members.clone());
        for witness in 0..10 {
            let value = atom(witness);
            assert_eq!(
                value.is_assignable_to(&raw),
                value.is_assignable_to(&normalized),
                "seed={NORMALIZATION_SEED:#x} case={case} members={members:?} witness={value:?}"
            );
        }
    }
}

fn finite_value(index: usize) -> Type {
    match index {
        0 => Type::None,
        1 => Type::LiteralInt(0),
        2 => Type::LiteralInt(7),
        3 => Type::LiteralStr(String::new()),
        4 => Type::LiteralStr("x".to_string()),
        5 => Type::LiteralBool(false),
        _ => Type::LiteralBool(true),
    }
}

fn condition(index: usize) -> NarrowingCondition {
    let x = "x".to_string();
    match index {
        0 => NarrowingCondition::IsNone(x),
        1 => NarrowingCondition::IsNotNone(x),
        2 => NarrowingCondition::IsInstance(x, Type::Int),
        3 => NarrowingCondition::IsInstance(x, Type::Str),
        4 => NarrowingCondition::IsInstance(x, Type::Bool),
        5 => NarrowingCondition::Equality(x, LiteralValue::Int(0)),
        6 => NarrowingCondition::Equality(x, LiteralValue::Str("x".to_string())),
        7 => NarrowingCondition::Truthiness(x),
        8 => NarrowingCondition::Not(Box::new(condition(0))),
        9 => NarrowingCondition::Not(Box::new(condition(2))),
        _ => NarrowingCondition::Not(Box::new(condition(7))),
    }
}

fn predicate_holds(value: &Type, index: usize) -> bool {
    match index {
        0 => *value == Type::None,
        1 => *value != Type::None,
        2 => matches!(value, Type::LiteralInt(_)),
        3 => matches!(value, Type::LiteralStr(_)),
        4 => matches!(value, Type::LiteralBool(_)),
        5 => *value == Type::LiteralInt(0),
        6 => *value == Type::LiteralStr("x".to_string()),
        7 => {
            !matches!(
                value,
                Type::None | Type::LiteralInt(0) | Type::LiteralBool(false)
            ) && !matches!(value, Type::LiteralStr(text) if text.is_empty())
        }
        8 => !predicate_holds(value, 0),
        9 => !predicate_holds(value, 2),
        _ => !predicate_holds(value, 7),
    }
}

#[test]
fn narrowing_partition() {
    let mut rng = Rng(NARROWING_SEED);
    for case in 0..CASES {
        // All seven singleton values are independently selected. This includes
        // absent/present None optionals, unions, and the empty domain.
        let mask = u8::try_from(rng.next() & 0x7f).expect("seven-bit mask fits u8");
        let members = (0..7)
            .filter(|index| mask & (1 << index) != 0)
            .map(finite_value)
            .collect::<Vec<_>>();
        let input = make_union(members);
        for condition_index in 0..11 {
            let cond = condition(condition_index);
            let yes = narrow_type(&input, &cond, true);
            let no = narrow_type(&input, &cond, false);
            for value_index in 0..7 {
                let value = finite_value(value_index);
                let in_input = value.is_assignable_to(&input);
                let expected_yes = in_input && predicate_holds(&value, condition_index);
                let expected_no = in_input && !predicate_holds(&value, condition_index);
                assert_eq!(
                    value.is_assignable_to(&yes),
                    expected_yes,
                    "seed={NARROWING_SEED:#x} case={case} mask={mask:#x} cond={cond:?} branch=true value={value:?} result={yes:?}"
                );
                assert_eq!(
                    value.is_assignable_to(&no),
                    expected_no,
                    "seed={NARROWING_SEED:#x} case={case} mask={mask:#x} cond={cond:?} branch=false value={value:?} result={no:?}"
                );
            }
        }
    }
}
