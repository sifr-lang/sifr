use std::convert::Infallible;
use std::str::FromStr;

use super::*;

fn alignment(c: u8) -> Option<Alignment> {
    match c {
        b'<' => Some(Alignment::Left),
        b'^' => Some(Alignment::Center),
        b'>' => Some(Alignment::Right),
        _ => None,
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Alignment {
    Left,
    Center,
    Right,
    #[default]
    None,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum Sign {
    Plus,
    Minus,
    #[default]
    None,
}

#[derive(Debug, Default, Clone, Copy)]
#[allow(non_camel_case_types)]
pub(crate) enum TraitSpec<'a> {
    #[default]
    Display,
    #[cfg(feature = "debug")]
    Question,
    #[cfg(feature = "debug")]
    LowerHexQuestion,
    #[cfg(feature = "debug")]
    UpperHexQuestion,
    #[cfg(feature = "number")]
    LowerHex,
    #[cfg(feature = "number")]
    UpperHex,
    #[cfg(feature = "number")]
    Binary,
    #[cfg(feature = "number")]
    Octal,
    #[cfg(feature = "number")]
    LowerExp,
    #[cfg(feature = "number")]
    UpperExp,
    #[cfg(feature = "pointer")]
    Pointer,
    #[cfg(feature = "iter")]
    Iter(Option<Range>, Option<&'a str>, Option<&'a str>),
    #[allow(unused)]
    Phantom(&'a Infallible),
}

fn parse_number<T: FromStr, IE: Into<Error>>(
    format: &mut &str,
    idx: &mut usize,
    start: usize,
    error: fn(<T as FromStr>::Err, usize) -> IE,
) -> Result<T> {
    let len = format
        .find(|c: char| c != '-' && !c.is_ascii_digit())
        .ok_or(ParseError::FormatSpecUnclosed(start))?;
    let i = format[..len].parse().map_err(|e| error(e, *idx).into())?;
    step(len, format, idx);
    Ok(i)
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Range(
    pub(crate) Option<isize>,
    pub(crate) bool,
    pub(crate) Option<isize>,
);

impl<'a> TraitSpec<'a> {
    #[allow(clippy::too_many_lines, unused_variables)]
    fn from_str(format: &mut &'a str, idx: &mut usize, start: usize) -> Result<Self> {
        if format.starts_with('}') {
            return Ok(Self::Display);
        }
        match format.as_bytes()[0] {
            #[cfg(feature = "debug")]
            b'?' => Ok(TraitSpec::Question),
            #[cfg(not(feature = "debug"))]
            b'?' => Err(Error::UnsupportedOption("?", "debug", *idx)),
            #[cfg(feature = "debug")]
            b'x' if format.as_bytes()[1] == b'?' => {
                step(1, format, idx);
                Ok(TraitSpec::LowerHexQuestion)
            }
            #[cfg(not(feature = "debug"))]
            b'x' if format.as_bytes()[1] == b'?' => {
                Err(Error::UnsupportedOption("x?", "debug", *idx))
            }
            #[cfg(feature = "debug")]
            b'X' if format.as_bytes()[1] == b'?' => {
                step(1, format, idx);
                Ok(TraitSpec::UpperHexQuestion)
            }
            #[cfg(not(feature = "debug"))]
            b'X' if format.as_bytes()[1] == b'?' => {
                Err(Error::UnsupportedOption("X?", "debug", *idx))
            }
            #[cfg(feature = "number")]
            b'o' => Ok(TraitSpec::Octal),
            #[cfg(not(feature = "number"))]
            b'o' => Err(Error::UnsupportedOption("o", "number", *idx)),
            #[cfg(feature = "number")]
            b'x' => Ok(TraitSpec::LowerHex),
            #[cfg(not(feature = "number"))]
            b'x' => Err(Error::UnsupportedOption("x", "number", *idx)),
            #[cfg(feature = "number")]
            b'X' => Ok(TraitSpec::UpperHex),
            #[cfg(not(feature = "number"))]
            b'X' => Err(Error::UnsupportedOption("X", "number", *idx)),
            #[cfg(feature = "number")]
            b'b' => Ok(TraitSpec::Binary),
            #[cfg(not(feature = "number"))]
            b'b' => Err(Error::UnsupportedOption("b", "number", *idx)),
            #[cfg(feature = "number")]
            b'e' => Ok(TraitSpec::LowerExp),
            #[cfg(not(feature = "number"))]
            b'e' => Err(Error::UnsupportedOption("e", "number", *idx)),
            #[cfg(feature = "number")]
            b'E' => Ok(TraitSpec::UpperExp),
            #[cfg(not(feature = "number"))]
            b'E' => Err(Error::UnsupportedOption("E", "number", *idx)),
            #[cfg(feature = "pointer")]
            b'p' => Ok(TraitSpec::Pointer),
            #[cfg(not(feature = "pointer"))]
            b'p' => Err(Error::UnsupportedOption("p", "pointer", *idx)),
            #[cfg(feature = "iter")]
            b'i' => {
                step(1, format, idx);
                return Ok(TraitSpec::Iter(
                    if format.starts_with('-')
                        || format.starts_with("..")
                        || format.starts_with(|c: char| c.is_ascii_digit())
                    {
                        let lhs = if format.starts_with("..") {
                            None
                        } else {
                            Some(parse_number(format, idx, start, ParseError::RangeBound)?)
                        };
                        let inclusive;
                        let rhs;
                        if format.starts_with("..") {
                            step(2, format, idx);
                            inclusive = format.starts_with('=');
                            if inclusive {
                                step(1, format, idx);
                            }
                            rhs = if format.starts_with('-')
                                || format.starts_with(|c: char| c.is_ascii_digit())
                            {
                                Some(parse_number(format, idx, start, ParseError::RangeBound)?)
                            } else {
                                None
                            }
                        } else {
                            inclusive = true;
                            rhs = lhs;
                        }
                        Some(Range(lhs, inclusive, rhs))
                    } else {
                        None
                    },
                    if format.starts_with('(') || format.starts_with('#') {
                        Some(collect_parenthesized(format, idx, start)?)
                    } else {
                        None
                    },
                    if format.starts_with('(') || format.starts_with('#') {
                        Some(collect_parenthesized(format, idx, start)?)
                    } else {
                        None
                    },
                ));
            }
            #[cfg(not(feature = "iter"))]
            b'i' => Err(Error::UnsupportedOption("i", "iter", *idx)),
            _ => Err(ParseError::Expected("}", *idx).into()),
        }
        .map(|m| {
            step(1, format, idx);
            m
        })
    }
}

#[cfg(feature = "iter")]
fn collect_parenthesized<'a>(
    format: &mut &'a str,
    idx: &mut usize,
    outer_start: usize,
) -> Result<&'a str> {
    let pounds = format
        .find(|c| c != '#')
        .ok_or(ParseError::FormatSpecUnclosed(outer_start))?;
    step(pounds, format, idx);
    ensure! {format.starts_with('('), ParseError::Expected("(", *idx)};
    step(1, format, idx);
    let start = *idx;
    let inner = *format;
    loop {
        if format.starts_with(')')
            && format.len() > pounds
            && format[1..].chars().take(pounds).all(|c| c == '#')
        {
            step(1 + pounds, format, idx);
            break;
        }
        step(
            format
                .chars()
                .next()
                .ok_or(ParseError::FormatSpecUnclosed(outer_start))? // TODO correct index
                .len_utf8(),
            format,
            idx,
        );
    }
    Ok(&inner[..*idx - start - 1 - pounds])
}

#[derive(Default, Debug)]
pub(crate) struct FormatArgument<'a> {
    pub(crate) ident: &'a str,
    pub(crate) alignment: Alignment,
    pub(crate) sign: Sign,
    pub(crate) hash: bool,
    pub(crate) zero: bool,
    pub(crate) width: Option<usize>,
    pub(crate) precision: Option<usize>,
    pub(crate) trait_: TraitSpec<'a>,
}

pub(crate) fn step(len: usize, format: &mut &str, idx: &mut usize) {
    *format = &format[len..];
    *idx += len;
}

impl<'a> FormatArgument<'a> {
    pub(crate) fn from_str(format: &mut &'a str, idx: &mut usize) -> Result<Self> {
        let start_index = *idx;
        let mut it = Self::default();
        step(
            format.find(|c: char| !c.is_whitespace()).unwrap_or(0),
            format,
            idx,
        );
        let variable_len = format
            .find(|c| c == ':' || c == '}')
            .ok_or(ParseError::FormatSpecUnclosed(start_index))?;
        it.ident = &format[..variable_len];
        step(variable_len, format, idx);
        if format.starts_with(':') {
            step(1, format, idx);
            if format.is_empty() {
                return Ok(it);
            }
            let fill = format
                .chars()
                .next()
                .ok_or(ParseError::Expected("}", *idx))?;
            if format[fill.len_utf8()..].is_empty() {
                return Ok(it);
            }
            if alignment(format[fill.len_utf8()..].as_bytes()[0]).is_some() {
                return Err(ParseError::Fill(*idx).into());
            } else if fill.is_ascii() {
                if let Some(alignment) = alignment(fill as u8) {
                    it.alignment = alignment;
                    step(1, format, idx);
                }
            }
            if format.is_empty() {
                return Ok(it);
            }
            match format.as_bytes()[0] {
                b'+' => {
                    step(1, format, idx);
                    it.sign = Sign::Plus;
                }
                b'-' => {
                    step(1, format, idx);
                    it.sign = Sign::Minus;
                }
                _ => {}
            }
            if format.is_empty() {
                return Ok(it);
            }
            if b'#' == format.as_bytes()[0] {
                step(1, format, idx);
                it.hash = true;
            }
            if format.starts_with('0') {
                it.zero = true;
                step(1, format, idx);
            }
            if format.starts_with(|c: char| c.is_ascii_digit()) {
                it.width = Some(parse_number(
                    format,
                    idx,
                    start_index,
                    ParseError::InvalidWidth,
                )?);
            }
            if format.starts_with('.') {
                step(1, format, idx);
                it.precision = Some(parse_number(
                    format,
                    idx,
                    start_index,
                    ParseError::InvalidPrecision,
                )?);
            }
            it.trait_ = TraitSpec::from_str(format, idx, start_index)?;
        }
        Ok(it)
    }
}

#[cfg(test)]
#[cfg(feature = "iter")]
mod test {
    use super::*;
    #[test]
    fn collect_parenthesized() {
        use super::collect_parenthesized;
        let mut idx = 0;
        let mut format = "()";
        assert_eq!(collect_parenthesized(&mut format, &mut idx, 0).unwrap(), "");
        assert_eq!(idx, 2);
        assert_eq!(format, "");
        let mut idx = 0;
        let mut format = "#(a)))# hello";
        assert_eq!(
            collect_parenthesized(&mut format, &mut idx, 0).unwrap(),
            "a))"
        );
        assert_eq!(idx, 7);
        assert_eq!(format, " hello");
        let mut idx = 0;
        let mut format = "###((a:)#>))##))###)### hello";
        assert_eq!(
            collect_parenthesized(&mut format, &mut idx, 0).unwrap(),
            "(a:)#>))##)"
        );
        assert_eq!(idx, 19);
        assert_eq!(format, ")### hello");

        assert_eq!(
            collect_parenthesized(&mut "###()##", &mut 0, 10).unwrap_err(),
            Error::Parse(ParseError::FormatSpecUnclosed(10))
        );
        assert_eq!(
            collect_parenthesized(&mut "###", &mut 0, 10).unwrap_err(),
            Error::Parse(ParseError::FormatSpecUnclosed(10))
        );
        assert_eq!(
            collect_parenthesized(&mut "###a", &mut 0, 10).unwrap_err(),
            Error::Parse(ParseError::Expected("(", 3))
        );
    }
}
