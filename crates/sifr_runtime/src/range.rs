use crate::SifrInt;
use num_bigint::BigInt;
use num_traits::Signed;

/// A lazy exact-integer range used for every source-level `range` value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SifrRange {
    front: SifrInt,
    back_exclusive: SifrInt,
    step: SifrInt,
}

impl SifrRange {
    /// Creates a range after the compiler has proved that `step` is non-zero.
    #[must_use]
    pub fn new_known_nonzero(start: SifrInt, end: SifrInt, step: SifrInt) -> Self {
        debug_assert!(!step.is_zero());
        let end = end.into_bigint();
        let count = range_element_count(&start.as_bigint(), &end, &step.as_bigint());
        let back_exclusive = SifrInt::from_bigint(start.as_bigint() + step.as_bigint() * count);
        Self {
            front: start,
            back_exclusive,
            step,
        }
    }

    #[must_use]
    pub fn contains(&self, value: &SifrInt) -> bool {
        let value = value.as_bigint();
        let front = self.front.as_bigint();
        let back = self.back_exclusive.as_bigint();
        let step = self.step.as_bigint();
        let in_bounds = if step.is_positive() {
            value >= front && value < back
        } else {
            value <= front && value > back
        };
        in_bounds && (value - front) % step == BigInt::ZERO
    }

    fn is_empty(&self) -> bool {
        self.front == self.back_exclusive
    }
}

impl Iterator for SifrRange {
    type Item = SifrInt;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }
        let value = self.front.clone();
        self.front = &self.front + &self.step;
        Some(value)
    }
}

impl DoubleEndedIterator for SifrRange {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }
        self.back_exclusive = &self.back_exclusive - &self.step;
        Some(self.back_exclusive.clone())
    }
}

fn range_element_count(start: &BigInt, end: &BigInt, step: &BigInt) -> BigInt {
    if step.is_positive() {
        if start >= end {
            return BigInt::ZERO;
        }
        ceil_div_positive(&(end - start), step)
    } else {
        if start <= end {
            return BigInt::ZERO;
        }
        ceil_div_positive(&(start - end), &(-step))
    }
}

fn ceil_div_positive(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    (numerator + denominator - BigInt::from(1_u8)) / denominator
}
