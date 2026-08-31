use crate::SifrInt;

const WIDTH_RANGE_ERROR: &str = "requested string width exceeds the addressable platform range";
const CAPACITY_ERROR: &str = "requested string width cannot be allocated";

pub fn checked_ljust(value: &str, width: &SifrInt) -> Result<String, String> {
    checked_pad(value, width, Alignment::Left)
}

pub fn checked_rjust(value: &str, width: &SifrInt) -> Result<String, String> {
    checked_pad(value, width, Alignment::Right)
}

pub fn checked_center(value: &str, width: &SifrInt) -> Result<String, String> {
    checked_pad(value, width, Alignment::Center)
}

pub fn checked_zfill(value: &str, width: &SifrInt) -> Result<String, String> {
    let width = normalized_width(width)?;
    let character_count = value.chars().count();
    let padding = width.saturating_sub(character_count);
    if padding == 0 {
        return checked_copy(value);
    }

    let mut output = reserved_string(value.len(), padding)?;
    let mut chars = value.chars();
    if let Some(sign @ ('+' | '-')) = chars.next() {
        output.push(sign);
        push_repeated(&mut output, '0', padding);
        output.extend(chars);
    } else {
        push_repeated(&mut output, '0', padding);
        output.push_str(value);
    }
    Ok(output)
}

#[derive(Clone, Copy)]
enum Alignment {
    Left,
    Right,
    Center,
}

fn checked_pad(value: &str, width: &SifrInt, alignment: Alignment) -> Result<String, String> {
    let width = normalized_width(width)?;
    let padding = width.saturating_sub(value.chars().count());
    if padding == 0 {
        return checked_copy(value);
    }

    let (left, right) = match alignment {
        Alignment::Left => (0, padding),
        Alignment::Right => (padding, 0),
        Alignment::Center => (padding / 2, padding - (padding / 2)),
    };
    let mut output = reserved_string(value.len(), padding)?;
    push_repeated(&mut output, ' ', left);
    output.push_str(value);
    push_repeated(&mut output, ' ', right);
    Ok(output)
}

fn normalized_width(width: &SifrInt) -> Result<usize, String> {
    if width.is_negative() {
        return Ok(0);
    }
    width
        .try_to_usize()
        .map_err(|_| WIDTH_RANGE_ERROR.to_string())
}

fn checked_copy(value: &str) -> Result<String, String> {
    let mut output = reserved_string(value.len(), 0)?;
    output.push_str(value);
    Ok(output)
}

fn reserved_string(value_bytes: usize, padding: usize) -> Result<String, String> {
    let capacity = value_bytes
        .checked_add(padding)
        .ok_or_else(|| CAPACITY_ERROR.to_string())?;
    let mut output = String::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_| CAPACITY_ERROR.to_string())?;
    Ok(output)
}

fn push_repeated(output: &mut String, value: char, count: usize) {
    for _ in 0..count {
        output.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::{checked_center, checked_ljust, checked_rjust, checked_zfill};
    use crate::SifrInt;

    #[test]
    fn padding_preserves_python_width_and_sign_semantics() {
        assert_eq!(checked_ljust("x", &SifrInt::from_i64(4)), Ok("x   ".into()));
        assert_eq!(checked_rjust("x", &SifrInt::from_i64(4)), Ok("   x".into()));
        assert_eq!(
            checked_center("x", &SifrInt::from_i64(4)),
            Ok(" x  ".into())
        );
        assert_eq!(
            checked_zfill("-7", &SifrInt::from_i64(4)),
            Ok("-007".into())
        );
        assert_eq!(
            checked_zfill("+7", &SifrInt::from_i64(4)),
            Ok("+007".into())
        );
        assert_eq!(checked_ljust("é", &SifrInt::from_i64(3)), Ok("é  ".into()));
        assert_eq!(checked_ljust("x", &SifrInt::from_i64(-1)), Ok("x".into()));
    }

    #[test]
    fn out_of_range_width_is_typed_failure() {
        let width = SifrInt::parse_decimal("100000000000000000000000000000", 40)
            .expect("fixture integer parses");
        assert!(checked_center("x", &width).is_err());
    }
}
