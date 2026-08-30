use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Cardinality {
    Empty,
    Interval { minimum: u64, maximum: Option<u64> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardinalityError {
    InvalidInterval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchMethod {
    Execute,
    FetchOne,
    FetchOptional,
    FetchAll,
    Stream,
}

impl Cardinality {
    pub const BOTTOM: Self = Self::Empty;
    pub const ZERO: Self = Self::Interval {
        minimum: 0,
        maximum: Some(0),
    };
    pub const AT_MOST_ONE: Self = Self::Interval {
        minimum: 0,
        maximum: Some(1),
    };
    pub const EXACTLY_ONE: Self = Self::Interval {
        minimum: 1,
        maximum: Some(1),
    };
    pub const ONE_OR_MORE: Self = Self::Interval {
        minimum: 1,
        maximum: None,
    };
    pub const MANY: Self = Self::Interval {
        minimum: 0,
        maximum: None,
    };

    pub fn new(minimum: u64, maximum: Option<u64>) -> Result<Self, CardinalityError> {
        if maximum.is_some_and(|maximum| minimum > maximum) {
            return Err(CardinalityError::InvalidInterval);
        }
        Ok(Self::Interval { minimum, maximum })
    }

    pub fn validate(self) -> Result<(), CardinalityError> {
        match self {
            Self::Empty => Ok(()),
            Self::Interval { minimum, maximum } => Self::new(minimum, maximum).map(|_| ()),
        }
    }

    #[must_use]
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Empty, value) | (value, Self::Empty) => value,
            (
                Self::Interval {
                    minimum: left_minimum,
                    maximum: left_maximum,
                },
                Self::Interval {
                    minimum: right_minimum,
                    maximum: right_maximum,
                },
            ) => Self::Interval {
                minimum: left_minimum.min(right_minimum),
                maximum: max_bound(left_maximum, right_maximum),
            },
        }
    }

    #[must_use]
    pub fn meet(self, other: Self) -> Self {
        match (self, other) {
            (Self::Empty, _) | (_, Self::Empty) => Self::Empty,
            (
                Self::Interval {
                    minimum: left_minimum,
                    maximum: left_maximum,
                },
                Self::Interval {
                    minimum: right_minimum,
                    maximum: right_maximum,
                },
            ) => {
                let minimum = left_minimum.max(right_minimum);
                let maximum = min_bound(left_maximum, right_maximum);
                if maximum.is_some_and(|maximum| minimum > maximum) {
                    Self::Empty
                } else {
                    Self::Interval { minimum, maximum }
                }
            }
        }
    }

    #[must_use]
    pub fn supports(self, method: FetchMethod, returns_rows: bool) -> bool {
        match method {
            FetchMethod::Execute => !returns_rows,
            FetchMethod::FetchOne | FetchMethod::FetchOptional => {
                returns_rows && self.upper_bound_is_at_most_one()
            }
            FetchMethod::FetchAll | FetchMethod::Stream => returns_rows,
        }
    }

    #[must_use]
    pub fn upper_bound_is_at_most_one(self) -> bool {
        matches!(
            self,
            Self::Empty
                | Self::Interval {
                    maximum: Some(0 | 1),
                    ..
                }
        )
    }
}

impl fmt::Display for CardinalityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("cardinality minimum exceeds its maximum")
    }
}

impl std::error::Error for CardinalityError {}

const fn max_bound(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(if left > right { left } else { right }),
        (None, _) | (_, None) => None,
    }
}

const fn min_bound(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(if left < right { left } else { right }),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}
