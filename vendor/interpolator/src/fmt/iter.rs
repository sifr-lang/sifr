use crate::*;

fn write_iter(
    out: &mut impl Write,
    value: &[Formattable<'_>],
    range: Option<Range>,
    format: Option<&str>,
    join: Option<&str>,
    context: &impl Context,
) -> Result {
    if value.is_empty() {
        return Ok(());
    }
    let Range(lhs, inclusive, rhs) = range.unwrap_or(Range(None, false, None));
    let rhs = rhs.unwrap_or(isize::MAX);
    let rhs = (usize::try_from(rhs).unwrap_or(value.len().saturating_sub(rhs.unsigned_abs()))
        + usize::from(inclusive))
    .min(value.len());
    let lhs = lhs.unwrap_or(0);
    let lhs = usize::try_from(lhs)
        .unwrap_or(value.len().saturating_sub(lhs.unsigned_abs()))
        .min(rhs);

    if rhs > lhs {
        if let Some(format) = format {
            for value in &value[lhs..rhs - 1] {
                write(out, format, &IterContext::new(context, *value))?;
                if let Some(join) = join {
                    write(out, join, context)?;
                }
            }
            if let Some(value) = value[..rhs].last() {
                write(out, format, &IterContext::new(context, *value))?;
            }
        } else {
            for value in &value[lhs..rhs] {
                write!(
                    out,
                    "{}",
                    value
                        .get_display()
                        .map_err(|e| Error::MissingTraitImpl(e, 0))?
                )
                .map_err(|e| Error::Fmt(e, 0))?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn iter(
    out: &mut impl Write,
    value: &[Formattable<'_>],
    width: Option<usize>,
    precision: Option<usize>,
    alignment: crate::Alignment,
    sign: Sign,
    hash: bool,
    zero: bool,
    range: Option<Range>,
    format: Option<&str>,
    join: Option<&str>,
    context: &impl Context,
) -> Result {
    match (precision, sign, hash, zero) {
        (None, Sign::None, false, false) => {
            if let Some(width) = width {
                let mut buf = String::new();
                write_iter(&mut buf, value, range, format, join, context)?;
                match alignment {
                    Alignment::Left | Alignment::None => write!(out, "{buf:<width$}"),
                    Alignment::Center => write!(out, "{buf:^width$}"),
                    Alignment::Right => write!(out, "{buf:>width$}"),
                }
                .map_err(|e| Error::Fmt(e, 0))
            } else {
                write_iter(out, value, range, format, join, context)
            }
        }
        _ => Err(ParseError::Iter(0).into()),
    }
}

#[cfg(test)]
mod test {
    use collection_literals::hash;

    use super::*;

    #[test]
    fn iter() {
        let list = &[&1, &5].map(Formattable::display);
        let context = &hash!("h"=> Formattable::iter(list));
        assert_eq!(
            format("{h:i(`{:+05}`)#() )#}", context).unwrap(),
            "`+0001`) `+0005`"
        );
        assert_eq!(format("{h:i(``)}", context).unwrap(), "````");
        assert_eq!(format("{h:i..({})}", context).unwrap(), "15");
        assert_eq!(format("{h:i1..({})}", context).unwrap(), "5");
        assert_eq!(format("{h:i1..1({})}", context).unwrap(), "");
        assert_eq!(format("{h:i2..1({})}", context).unwrap(), "");
        assert_eq!(format("{h:i-1..({})}", context).unwrap(), "5");
        assert_eq!(format("{h:i..-1({})}", context).unwrap(), "1");
        assert_eq!(format("{h:i..-2({})}", context).unwrap(), "");
        assert_eq!(format("{h:i-5..-10({})}", context).unwrap(), "");
        assert_eq!(format("{h:i-1({})}", context).unwrap(), "5");
        assert_eq!(format("{h:i1}", context).unwrap(), "5");
        assert_eq!(format("{h:i..}", context).unwrap(), "15");
        assert_eq!(format("{h:i}", context).unwrap(), "15");
        assert_eq!(format("{h:i..1}", context).unwrap(), "1");
        assert_eq!(format("{h:i1..}", context).unwrap(), "5");
        assert_eq!(format("{h:i..=-1}", context).unwrap(), "15");
    }
    #[test]
    fn outside_context() {
        let list = &[&"hi", &"ho"].map(Formattable::display);
        let context = &hash!("list" => Formattable::iter(list), "x" => Formattable::display(&"?!"));
        assert_eq!(format("{list:i({}{x})}", context).unwrap(), "hi?!ho?!");
        assert_eq!(format("{list:i({})({x})}", context).unwrap(), "hi?!ho");
    }
}
