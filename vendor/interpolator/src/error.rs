use super::*;

/// Error returned by [`format()`].
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum Error {
    /// Value was formatted with unimplemented trait.
    /// - `.0` the trait
    /// - `.1` the byte index of the format argument
    MissingTraitImpl(Trait, usize),
    /// Error occurred while calling `::fmt`
    /// - `.0` the error
    /// - `.1` the byte index of the format argument
    Fmt(FmtError, usize),
    /// Error occurred  while parsing format string
    Parse(ParseError),
    /// Tried to format value that was not in context
    /// - `.0` the ident of the value
    /// - `.1` the byte index of the format argument
    MissingValue(String, usize),
    /// Unsupported Option was used.
    /// - `.0` is the option
    /// - `.1` the feature that needs to be enabled to use it
    /// - `.2` the byte index of the format argument
    UnsupportedOption(&'static str, &'static str, usize),
}

#[cfg(feature = "iter")]
impl Error {
    pub(crate) fn add_idx(self, idx: usize) -> Self {
        use Error::*;
        match self {
            MissingTraitImpl(t, i) => MissingTraitImpl(t, i + idx),
            Fmt(e, i) => Fmt(e, i),
            Parse(e) => Parse(e.add_idx(idx)),
            MissingValue(v, i) => MissingValue(v, i + idx),
            UnsupportedOption(o, f, i) => UnsupportedOption(o, f, i + idx),
        }
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingTraitImpl(t, idx) => write!(
                f,
                "trait `{t:?}` not implemented, used by format argument at {idx}"
            ),
            Error::Fmt(e, idx) => write!(f, "error while formatting at {idx}: {e}"),
            Error::Parse(e) => write!(f, "error while parsing input: {e}"),
            Error::MissingValue(ident, idx) => {
                write!(f, "specified value not in context `{ident}` at {idx}")
            }
            Error::UnsupportedOption(option, feature, idx) => write!(
                f,
                "option `{option}` is not supported without feature `{feature}` at {idx}"
            ),
        }
    }
}

impl StdError for Error {}

/// The trait used to format.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum Trait {
    /// [`Binary`]
    #[cfg(feature = "number")]
    Binary,
    /// [`Debug`]
    #[cfg(feature = "debug")]
    Debug,
    /// [`Display`]
    Display,
    /// [`LowerExp`]
    #[cfg(feature = "number")]
    LowerExp,
    /// [`LowerHex`]
    #[cfg(feature = "number")]
    LowerHex,
    /// [`Octal`]
    #[cfg(feature = "number")]
    Octal,
    /// [`Pointer`]
    #[cfg(feature = "pointer")]
    Pointer,
    /// [`UpperExp`]
    #[cfg(feature = "number")]
    UpperExp,
    /// [`UpperHex`]
    #[cfg(feature = "number")]
    UpperHex,
    #[cfg(feature = "iter")]
    /// Custom "trait" that allows iterating over lists i.e. `&[Formattable]`
    Iter,
}

/// Error caused by invalid format string
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum ParseError {
    /// Format spec at byte index is nether closed with a `}`
    FormatSpecUnclosed(usize),
    /// Expected sequence at byte index
    Expected(&'static str, usize),
    /// Unable to parse specified width as usize
    InvalidWidth(ParseIntError, usize),
    /// Unable to parse specified precision as usize
    InvalidPrecision(ParseIntError, usize),
    /// Fill is not supported due to [rust-lang/rfcs#3394](https://github.com/rust-lang/rfcs/pull/3394).
    Fill(usize),
    /// Width, precision, `-`, `+` and `#` are not supported for `i`
    Iter(usize),
    /// Unable to parse specified range bound as isize
    RangeBound(ParseIntError, usize),
}

#[cfg(feature = "iter")]
impl ParseError {
    pub(crate) fn add_idx(self, idx: usize) -> ParseError {
        use ParseError::*;
        match self {
            FormatSpecUnclosed(i) => FormatSpecUnclosed(i + idx),
            Expected(c, i) => Expected(c, i + idx),
            InvalidWidth(e, i) => InvalidWidth(e, i + idx),
            InvalidPrecision(e, i) => InvalidPrecision(e, i + idx),
            Fill(i) => Fill(i + idx),
            Iter(i) => Iter(i + idx),
            RangeBound(e, i) => RangeBound(e, i + idx),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::FormatSpecUnclosed(idx) => {
                write!(f, "Format spec at {idx} is nether closed with a `}}`")
            }
            ParseError::Expected(c, idx) => write!(f, "Expected `{c}` at {idx}"),
            ParseError::InvalidWidth(e, idx) => {
                write!(f, "Unable to parse width at {idx} as usize: {e}")
            }
            ParseError::InvalidPrecision(e, idx) => {
                write!(f, "Unable to parse precision at {idx} as usize: {e}")
            }
            ParseError::Fill(idx) => write!(
                f,
                "Fill is not supported due to https://github.com/rust-lang/rfcs/pull/3394 at {idx}"
            ),
            ParseError::Iter(idx) => write!(
                f,
                "Width, precision, `-`, `+` and `#` are not supported for `i` at {idx}"
            ),
            ParseError::RangeBound(e, idx) => {
                write!(f, "Unable to parse range bound at {idx} as isize: {e}")
            }
        }
    }
}

impl StdError for ParseError {}

macro_rules! ensure {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($error.into());
        }
    };
}
