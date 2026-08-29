use crate::SifrInt;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::iter::FusedIterator;

/// Exact Python-compatible indices for a nonzero-step slice over an addressable sequence.
#[derive(Clone, Debug)]
pub struct SifrSliceIndices {
    next: Option<usize>,
    stop: Option<usize>,
    step: Option<usize>,
    reverse: bool,
}

impl SifrSliceIndices {
    /// Constructs indices after the compiler has proved that `step` is nonzero.
    #[must_use]
    pub fn new_known_nonzero(
        len: usize,
        start: Option<&SifrInt>,
        stop: Option<&SifrInt>,
        step: &SifrInt,
    ) -> Self {
        assert!(
            !step.is_zero(),
            "compiler nonzero slice-step proof was invalid"
        );

        if step.is_negative() {
            let next = start.map_or_else(|| len.checked_sub(1), |value| reverse_bound(value, len));
            let stop = stop.and_then(|value| reverse_bound(value, len));
            Self {
                next,
                stop,
                step: (-step.as_bigint()).to_usize(),
                reverse: true,
            }
        } else {
            Self {
                next: Some(start.map_or(0, |value| value.clamp_slice_bound(len))),
                stop: Some(stop.map_or(len, |value| value.clamp_slice_bound(len))),
                step: step.try_to_usize().ok(),
                reverse: false,
            }
        }
    }
}

impl Iterator for SifrSliceIndices {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        if self.reverse {
            if self.stop.is_some_and(|stop| current <= stop) {
                self.next = None;
                return None;
            }
            self.next = self.step.and_then(|step| current.checked_sub(step));
        } else {
            let stop = self.stop.unwrap_or(0);
            if current >= stop {
                self.next = None;
                return None;
            }
            self.next = self.step.and_then(|step| current.checked_add(step));
        }
        Some(current)
    }
}

impl FusedIterator for SifrSliceIndices {}

fn reverse_bound(value: &SifrInt, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let len_exact = BigInt::from(len);
    let normalized = if value.is_negative() {
        value.as_bigint() + &len_exact
    } else {
        value.as_bigint()
    };
    if normalized < BigInt::ZERO {
        return None;
    }
    if normalized >= len_exact {
        return len.checked_sub(1);
    }
    Some(SifrInt::from_bigint(normalized).to_usize_proven_in_bounds())
}

#[cfg(test)]
mod tests {
    use super::SifrSliceIndices;
    use crate::SifrInt;

    fn exact(value: i64) -> SifrInt {
        SifrInt::from_i64(value)
    }

    #[test]
    fn forward_and_reverse_indices_match_python_bound_rules() {
        assert_eq!(
            SifrSliceIndices::new_known_nonzero(6, None, None, &exact(2)).collect::<Vec<_>>(),
            vec![0, 2, 4]
        );
        assert_eq!(
            SifrSliceIndices::new_known_nonzero(6, None, None, &exact(-2)).collect::<Vec<_>>(),
            vec![5, 3, 1]
        );
        assert_eq!(
            SifrSliceIndices::new_known_nonzero(6, Some(&exact(-5)), Some(&exact(-1)), &exact(2),)
                .collect::<Vec<_>>(),
            vec![1, 3]
        );
    }

    #[test]
    fn oversized_bounds_and_steps_are_clamped_without_narrowing() {
        let huge = SifrInt::from_decimal_literal("100000000000000000000000000000000000000");
        let negative_huge = -&huge;
        assert_eq!(
            SifrSliceIndices::new_known_nonzero(4, Some(&negative_huge), Some(&huge), &huge,)
                .collect::<Vec<_>>(),
            vec![0]
        );
        assert_eq!(
            SifrSliceIndices::new_known_nonzero(4, None, None, &negative_huge).collect::<Vec<_>>(),
            vec![3]
        );
    }
}
