//! Gettext `.mo` compatibility backend for native Sifr translation bundles.

use std::collections::BTreeMap;

use crate::encoding;

const MO_MAGIC_LE: [u8; 4] = [0xDE, 0x12, 0x04, 0x95];
const MO_MAGIC_BE: [u8; 4] = [0x95, 0x04, 0x12, 0xDE];
const HEADER_LEN: usize = 28;
const MAX_PLURAL_FORMS: usize = 32;
const MAX_PLURAL_EXPR_LEN: usize = 1_000;
const MAX_PLURAL_DEPTH: usize = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Endian {
    Little,
    Big,
}

#[derive(Debug)]
pub(crate) struct Catalog {
    messages: BTreeMap<String, Vec<String>>,
    plural: PluralFormula,
}

impl Catalog {
    pub(crate) fn parse(data: &[u8]) -> Result<Self, String> {
        let raw = RawCatalog::parse(data)?;
        let charset = raw.charset();
        let metadata = raw.metadata(&charset)?;
        let plural = PluralFormula::from_metadata(metadata.as_deref())?;
        let mut messages = BTreeMap::new();
        for entry in raw.entries {
            let original = decode_catalog_text(entry.original, &charset)?;
            if original.is_empty() {
                continue;
            }
            let translated = decode_catalog_text(entry.translated, &charset)?;
            messages.insert(
                message_key_from_original(&original),
                split_forms(&translated),
            );
        }
        Ok(Self { messages, plural })
    }

    pub(crate) fn lookup(&self, context: Option<&str>, message_id: &str) -> Option<String> {
        self.messages
            .get(&message_key(context, message_id))
            .and_then(|forms| non_empty_form(forms, 0))
    }

    pub(crate) fn lookup_plural(
        &self,
        context: Option<&str>,
        singular: &str,
        plural: &str,
        count: i64,
    ) -> Result<Option<String>, String> {
        let plural_key = format!("{singular}\0{plural}");
        let Some(forms) = self.messages.get(&message_key(context, &plural_key)) else {
            return Ok(None);
        };
        let index = self.plural.select(count)?;
        if index >= forms.len() {
            return Err(format!(
                "catalog plural form {index} is missing; catalog contains {} forms",
                forms.len()
            ));
        }
        Ok(non_empty_form(forms, index))
    }
}

fn non_empty_form(forms: &[String], index: usize) -> Option<String> {
    let form = forms.get(index)?;
    if form.is_empty() {
        None
    } else {
        Some(form.clone())
    }
}

#[derive(Debug)]
struct RawCatalog<'a> {
    entries: Vec<RawEntry<'a>>,
}

impl<'a> RawCatalog<'a> {
    fn parse(data: &'a [u8]) -> Result<Self, String> {
        if data.len() < HEADER_LEN {
            return Err(".mo header is truncated".to_string());
        }
        let endian = match data.get(0..4) {
            Some(bytes) if bytes == MO_MAGIC_LE => Endian::Little,
            Some(bytes) if bytes == MO_MAGIC_BE => Endian::Big,
            _ => return Err("invalid .mo magic number".to_string()),
        };
        let version = read_u32(data, 4, endian)?;
        if version >> 16 != 0 {
            return Err(format!("unsupported .mo major version {}", version >> 16));
        }
        let count = read_usize(data, 8, endian)?;
        let original_table = read_usize(data, 12, endian)?;
        let translated_table = read_usize(data, 16, endian)?;
        let original_table_end = table_end(original_table, count)?;
        let translated_table_end = table_end(translated_table, count)?;
        if original_table_end > data.len() || translated_table_end > data.len() {
            return Err(".mo string table extends past end of file".to_string());
        }
        let mut entries = Vec::with_capacity(count);
        for index in 0..count {
            let original_len = read_table_len(data, original_table, index, endian)?;
            let original_offset = read_table_offset(data, original_table, index, endian)?;
            let translated_len = read_table_len(data, translated_table, index, endian)?;
            let translated_offset = read_table_offset(data, translated_table, index, endian)?;
            entries.push(RawEntry {
                original: read_slice(data, original_offset, original_len, "original string")?,
                translated: read_slice(
                    data,
                    translated_offset,
                    translated_len,
                    "translated string",
                )?,
            });
        }
        Ok(Self { entries })
    }

    fn charset(&self) -> String {
        self.entries
            .iter()
            .find(|entry| entry.original.is_empty())
            .and_then(|entry| charset_from_metadata_bytes(entry.translated))
            .unwrap_or_else(|| "utf-8".to_string())
    }

    fn metadata(&self, charset: &str) -> Result<Option<String>, String> {
        self.entries
            .iter()
            .find(|entry| entry.original.is_empty())
            .map(|entry| decode_catalog_text(entry.translated, charset))
            .transpose()
    }
}

#[derive(Debug)]
struct RawEntry<'a> {
    original: &'a [u8],
    translated: &'a [u8],
}

fn table_end(offset: usize, count: usize) -> Result<usize, String> {
    count
        .checked_mul(8)
        .and_then(|bytes| offset.checked_add(bytes))
        .ok_or_else(|| ".mo string table offset overflow".to_string())
}

fn read_table_len(
    data: &[u8],
    table: usize,
    index: usize,
    endian: Endian,
) -> Result<usize, String> {
    let offset = index
        .checked_mul(8)
        .and_then(|bytes| table.checked_add(bytes))
        .ok_or_else(|| ".mo string table entry offset overflow".to_string())?;
    read_usize(data, offset, endian)
}

fn read_table_offset(
    data: &[u8],
    table: usize,
    index: usize,
    endian: Endian,
) -> Result<usize, String> {
    let offset = index
        .checked_mul(8)
        .and_then(|bytes| table.checked_add(bytes))
        .and_then(|base| base.checked_add(4))
        .ok_or_else(|| ".mo string table entry offset overflow".to_string())?;
    read_usize(data, offset, endian)
}

fn read_u32(data: &[u8], offset: usize, endian: Endian) -> Result<u32, String> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| ".mo header offset overflow".to_string())?;
    let bytes = data
        .get(offset..end)
        .ok_or_else(|| ".mo header is truncated".to_string())?;
    let array: [u8; 4] = bytes
        .try_into()
        .map_err(|_| ".mo header is truncated".to_string())?;
    Ok(match endian {
        Endian::Little => u32::from_le_bytes(array),
        Endian::Big => u32::from_be_bytes(array),
    })
}

fn read_usize(data: &[u8], offset: usize, endian: Endian) -> Result<usize, String> {
    usize::try_from(read_u32(data, offset, endian)?)
        .map_err(|_| ".mo offset does not fit this platform".to_string())
}

fn read_slice<'a>(
    data: &'a [u8],
    offset: usize,
    len: usize,
    label: &str,
) -> Result<&'a [u8], String> {
    let end = offset
        .checked_add(len)
        .ok_or_else(|| format!(".mo {label} offset overflow"))?;
    data.get(offset..end)
        .ok_or_else(|| format!(".mo {label} extends past end of file"))
}

fn charset_from_metadata_bytes(metadata: &[u8]) -> Option<String> {
    let lossy = String::from_utf8_lossy(metadata);
    for line in lossy.lines() {
        let lower = line.to_ascii_lowercase();
        let Some(pos) = lower.find("charset=") else {
            continue;
        };
        let value = &line[pos + "charset=".len()..];
        let label = value
            .split(|ch: char| ch == ';' || ch.is_ascii_whitespace())
            .next()
            .unwrap_or("")
            .trim();
        if !label.is_empty() {
            return Some(label.to_string());
        }
    }
    None
}

fn decode_catalog_text(data: &[u8], charset: &str) -> Result<String, String> {
    encoding::decode_text(data, charset, "strict")
        .map_err(|err| format!("failed to decode .mo catalog text as {charset}: {err}"))
}

fn message_key_from_original(original: &str) -> String {
    original.to_string()
}

fn message_key(context: Option<&str>, message_id: &str) -> String {
    match context {
        Some(value) => format!("{value}\u{4}{message_id}"),
        None => message_id.to_string(),
    }
}

fn split_forms(translated: &str) -> Vec<String> {
    translated.split('\0').map(str::to_string).collect()
}

#[derive(Debug)]
struct PluralFormula {
    nplurals: usize,
    expr: Expr,
}

impl PluralFormula {
    fn from_metadata(metadata: Option<&str>) -> Result<Self, String> {
        let Some(metadata) = metadata else {
            return Self::default_english();
        };
        let Some(line) = metadata.lines().find(|line| {
            line.split_once(':')
                .is_some_and(|(key, _)| key.trim().eq_ignore_ascii_case("Plural-Forms"))
        }) else {
            return Self::default_english();
        };
        let (_, value) = line
            .split_once(':')
            .ok_or_else(|| "malformed Plural-Forms header".to_string())?;
        let mut nplurals = None;
        let mut plural_expr = None;
        for part in value.split(';') {
            let Some((key, raw_value)) = part.split_once('=') else {
                continue;
            };
            match key.trim() {
                "nplurals" => {
                    let parsed = raw_value
                        .trim()
                        .parse::<usize>()
                        .map_err(|err| format!("invalid nplurals value: {err}"))?;
                    if parsed == 0 || parsed > MAX_PLURAL_FORMS {
                        return Err(format!("nplurals must be between 1 and {MAX_PLURAL_FORMS}"));
                    }
                    nplurals = Some(parsed);
                }
                "plural" => plural_expr = Some(raw_value.trim().to_string()),
                _ => {}
            }
        }
        let nplurals = nplurals.ok_or_else(|| "Plural-Forms missing nplurals".to_string())?;
        let plural_expr = plural_expr.ok_or_else(|| "Plural-Forms missing plural".to_string())?;
        let expr = Parser::new(&plural_expr)?.parse()?;
        Ok(Self { nplurals, expr })
    }

    fn default_english() -> Result<Self, String> {
        Ok(Self {
            nplurals: 2,
            expr: Parser::new("n != 1")?.parse()?,
        })
    }

    fn select(&self, count: i64) -> Result<usize, String> {
        let value = self.expr.eval(count)?;
        let index = usize::try_from(value)
            .map_err(|_| format!("plural expression selected negative form {value}"))?;
        if index >= self.nplurals {
            return Err(format!(
                "plural expression selected form {index}, but nplurals is {}",
                self.nplurals
            ));
        }
        Ok(index)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Expr {
    Number(i64),
    N,
    UnaryNot(Box<Expr>),
    UnaryNeg(Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
}

impl Expr {
    fn eval(&self, n: i64) -> Result<i64, String> {
        match self {
            Self::Number(value) => Ok(*value),
            Self::N => Ok(n),
            Self::UnaryNot(expr) => Ok(i64::from(expr.eval(n)? == 0)),
            Self::UnaryNeg(expr) => expr
                .eval(n)?
                .checked_neg()
                .ok_or_else(|| "plural expression overflow".to_string()),
            Self::Binary(op, left, right) => op.eval(left.eval(n)?, right.eval(n)?),
            Self::Conditional(condition, if_true, if_false) => {
                if condition.eval(n)? != 0 {
                    if_true.eval(n)
                } else {
                    if_false.eval(n)
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BinaryOp {
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

impl BinaryOp {
    const fn precedence(self) -> u8 {
        match self {
            Self::Or => 1,
            Self::And => 2,
            Self::Eq | Self::Ne => 3,
            Self::Lt | Self::Le | Self::Gt | Self::Ge => 4,
            Self::Add | Self::Sub => 5,
            Self::Mul | Self::Div | Self::Rem => 6,
        }
    }

    fn eval(self, left: i64, right: i64) -> Result<i64, String> {
        match self {
            Self::Or => Ok(i64::from(left != 0 || right != 0)),
            Self::And => Ok(i64::from(left != 0 && right != 0)),
            Self::Eq => Ok(i64::from(left == right)),
            Self::Ne => Ok(i64::from(left != right)),
            Self::Lt => Ok(i64::from(left < right)),
            Self::Le => Ok(i64::from(left <= right)),
            Self::Gt => Ok(i64::from(left > right)),
            Self::Ge => Ok(i64::from(left >= right)),
            Self::Add => left
                .checked_add(right)
                .ok_or_else(|| "plural expression overflow".to_string()),
            Self::Sub => left
                .checked_sub(right)
                .ok_or_else(|| "plural expression overflow".to_string()),
            Self::Mul => left
                .checked_mul(right)
                .ok_or_else(|| "plural expression overflow".to_string()),
            Self::Div => {
                if right == 0 {
                    return Err("plural expression division by zero".to_string());
                }
                left.checked_div(right)
                    .ok_or_else(|| "plural expression overflow".to_string())
            }
            Self::Rem => {
                if right == 0 {
                    return Err("plural expression remainder by zero".to_string());
                }
                left.checked_rem(right)
                    .ok_or_else(|| "plural expression overflow".to_string())
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Token {
    Number(i64),
    N,
    LParen,
    RParen,
    Question,
    Colon,
    Op(BinaryOp),
    Bang,
    Minus,
    Plus,
    End,
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    fn new(source: &str) -> Result<Self, String> {
        if source.len() > MAX_PLURAL_EXPR_LEN {
            return Err("plural expression is too long".to_string());
        }
        Ok(Self {
            tokens: tokenize(source)?,
            cursor: 0,
        })
    }

    fn parse(mut self) -> Result<Expr, String> {
        let expr = self.parse_conditional(0)?;
        if !matches!(self.peek(), Token::End) {
            return Err(format!(
                "unexpected token in plural expression: {:?}",
                self.peek()
            ));
        }
        Ok(expr)
    }

    fn parse_conditional(&mut self, depth: usize) -> Result<Expr, String> {
        if depth > MAX_PLURAL_DEPTH {
            return Err("plural expression is too complex".to_string());
        }
        let condition = self.parse_binary(1, depth)?;
        if !matches!(self.peek(), Token::Question) {
            return Ok(condition);
        }
        self.advance();
        let if_true = self.parse_conditional(depth + 1)?;
        if !matches!(self.peek(), Token::Colon) {
            return Err("plural conditional missing ':'".to_string());
        }
        self.advance();
        let if_false = self.parse_conditional(depth + 1)?;
        Ok(Expr::Conditional(
            Box::new(condition),
            Box::new(if_true),
            Box::new(if_false),
        ))
    }

    fn parse_binary(&mut self, min_precedence: u8, depth: usize) -> Result<Expr, String> {
        let mut left = self.parse_unary(depth)?;
        while let Some(op) = binary_op_from_token(&self.peek()) {
            let precedence = op.precedence();
            if precedence < min_precedence {
                break;
            }
            self.advance();
            let right = self.parse_binary(precedence + 1, depth + 1)?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self, depth: usize) -> Result<Expr, String> {
        if depth > MAX_PLURAL_DEPTH {
            return Err("plural expression is too complex".to_string());
        }
        match self.peek() {
            Token::Bang => {
                self.advance();
                Ok(Expr::UnaryNot(Box::new(self.parse_unary(depth + 1)?)))
            }
            Token::Minus => {
                self.advance();
                Ok(Expr::UnaryNeg(Box::new(self.parse_unary(depth + 1)?)))
            }
            Token::Plus => {
                self.advance();
                self.parse_unary(depth + 1)
            }
            Token::Number(value) => {
                self.advance();
                Ok(Expr::Number(value))
            }
            Token::N => {
                self.advance();
                Ok(Expr::N)
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_conditional(depth + 1)?;
                if !matches!(self.peek(), Token::RParen) {
                    return Err("unbalanced parenthesis in plural expression".to_string());
                }
                self.advance();
                Ok(expr)
            }
            other => Err(format!("unexpected token in plural expression: {other:?}")),
        }
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.cursor).cloned().unwrap_or(Token::End)
    }

    fn advance(&mut self) {
        self.cursor = self.cursor.saturating_add(1);
    }
}

fn binary_op_from_token(token: &Token) -> Option<BinaryOp> {
    match token {
        Token::Op(op) => Some(*op),
        Token::Plus => Some(BinaryOp::Add),
        Token::Minus => Some(BinaryOp::Sub),
        _ => None,
    }
}

fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if byte.is_ascii_whitespace() {
            cursor += 1;
            continue;
        }
        if byte.is_ascii_digit() {
            let start = cursor;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
            let raw = source
                .get(start..cursor)
                .ok_or_else(|| "invalid plural expression number".to_string())?;
            let value = raw
                .parse::<i64>()
                .map_err(|err| format!("invalid plural expression number: {err}"))?;
            tokens.push(Token::Number(value));
            continue;
        }
        match byte {
            b'n' => {
                tokens.push(Token::N);
                cursor += 1;
            }
            b'(' => {
                tokens.push(Token::LParen);
                cursor += 1;
            }
            b')' => {
                tokens.push(Token::RParen);
                cursor += 1;
            }
            b'?' => {
                tokens.push(Token::Question);
                cursor += 1;
            }
            b':' => {
                tokens.push(Token::Colon);
                cursor += 1;
            }
            b'!' if bytes.get(cursor + 1) == Some(&b'=') => {
                tokens.push(Token::Op(BinaryOp::Ne));
                cursor += 2;
            }
            b'!' => {
                tokens.push(Token::Bang);
                cursor += 1;
            }
            b'=' if bytes.get(cursor + 1) == Some(&b'=') => {
                tokens.push(Token::Op(BinaryOp::Eq));
                cursor += 2;
            }
            b'<' if bytes.get(cursor + 1) == Some(&b'=') => {
                tokens.push(Token::Op(BinaryOp::Le));
                cursor += 2;
            }
            b'<' => {
                tokens.push(Token::Op(BinaryOp::Lt));
                cursor += 1;
            }
            b'>' if bytes.get(cursor + 1) == Some(&b'=') => {
                tokens.push(Token::Op(BinaryOp::Ge));
                cursor += 2;
            }
            b'>' => {
                tokens.push(Token::Op(BinaryOp::Gt));
                cursor += 1;
            }
            b'&' if bytes.get(cursor + 1) == Some(&b'&') => {
                tokens.push(Token::Op(BinaryOp::And));
                cursor += 2;
            }
            b'|' if bytes.get(cursor + 1) == Some(&b'|') => {
                tokens.push(Token::Op(BinaryOp::Or));
                cursor += 2;
            }
            b'+' => {
                tokens.push(Token::Plus);
                cursor += 1;
            }
            b'-' => {
                tokens.push(Token::Minus);
                cursor += 1;
            }
            b'*' => {
                tokens.push(Token::Op(BinaryOp::Mul));
                cursor += 1;
            }
            b'/' => {
                tokens.push(Token::Op(BinaryOp::Div));
                cursor += 1;
            }
            b'%' => {
                tokens.push(Token::Op(BinaryOp::Rem));
                cursor += 1;
            }
            _ => {
                return Err(format!(
                    "invalid token in plural expression at byte offset {cursor}"
                ));
            }
        }
    }
    tokens.push(Token::End);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::{Catalog, Parser};

    fn mo_bytes(entries: &[(&[u8], &[u8])], big_endian: bool) -> Vec<u8> {
        let count = entries.len();
        let original_table = 28usize;
        let translated_table = original_table + count * 8;
        let mut cursor = translated_table + count * 8;
        let mut original_records = Vec::new();
        let mut translated_records = Vec::new();
        let mut payload = Vec::new();
        for (original, translated) in entries {
            original_records.push((*original, cursor));
            payload.extend_from_slice(original);
            payload.push(0);
            cursor += original.len() + 1;
            translated_records.push((*translated, cursor));
            payload.extend_from_slice(translated);
            payload.push(0);
            cursor += translated.len() + 1;
        }

        let mut data = Vec::new();
        if big_endian {
            data.extend_from_slice(&[0x95, 0x04, 0x12, 0xDE]);
        } else {
            data.extend_from_slice(&[0xDE, 0x12, 0x04, 0x95]);
        }
        push_u32(&mut data, 0, big_endian);
        push_u32(
            &mut data,
            u32::try_from(count).expect("test catalog entry count should fit u32"),
            big_endian,
        );
        push_u32(
            &mut data,
            u32::try_from(original_table).expect("test original table offset should fit u32"),
            big_endian,
        );
        push_u32(
            &mut data,
            u32::try_from(translated_table).expect("test translated table offset should fit u32"),
            big_endian,
        );
        push_u32(&mut data, 0, big_endian);
        push_u32(&mut data, 0, big_endian);
        for (text, offset) in original_records {
            push_u32(
                &mut data,
                u32::try_from(text.len()).expect("test original length should fit u32"),
                big_endian,
            );
            push_u32(
                &mut data,
                u32::try_from(offset).expect("test original offset should fit u32"),
                big_endian,
            );
        }
        for (text, offset) in translated_records {
            push_u32(
                &mut data,
                u32::try_from(text.len()).expect("test translation length should fit u32"),
                big_endian,
            );
            push_u32(
                &mut data,
                u32::try_from(offset).expect("test translation offset should fit u32"),
                big_endian,
            );
        }
        data.extend_from_slice(&payload);
        data
    }

    fn push_u32(data: &mut Vec<u8>, value: u32, big_endian: bool) {
        if big_endian {
            data.extend_from_slice(&value.to_be_bytes());
        } else {
            data.extend_from_slice(&value.to_le_bytes());
        }
    }

    #[test]
    fn mo_catalog_supports_context_plural_and_declared_charset() {
        let metadata = b"Content-Type: text/plain; charset=latin-1\nPlural-Forms: nplurals=2; plural=n != 1;\n";
        let data = mo_bytes(
            &[
                (b"", metadata),
                (b"hello", b"bonjour"),
                (b"empty", b""),
                (b"menu\x04open", b"ouvrir"),
                (b"file\x00files", b"fichier\x00fichiers"),
                (b"cafe", b"caf\xe9"),
            ],
            false,
        );
        let catalog = Catalog::parse(&data).expect("catalog should parse");

        assert_eq!(catalog.lookup(None, "hello").as_deref(), Some("bonjour"));
        assert_eq!(catalog.lookup(None, "empty"), None);
        assert_eq!(
            catalog.lookup(Some("menu"), "open").as_deref(),
            Some("ouvrir")
        );
        assert_eq!(
            catalog
                .lookup_plural(None, "file", "files", 1)
                .expect("plural lookup")
                .as_deref(),
            Some("fichier")
        );
        assert_eq!(
            catalog
                .lookup_plural(None, "file", "files", 2)
                .expect("plural lookup")
                .as_deref(),
            Some("fichiers")
        );
        assert_eq!(catalog.lookup(None, "cafe").as_deref(), Some("caf\u{e9}"));
    }

    #[test]
    fn mo_catalog_supports_big_endian_tables() {
        let data = mo_bytes(&[(b"foo", b"bar")], true);
        let catalog = Catalog::parse(&data).expect("big-endian catalog should parse");

        assert_eq!(catalog.lookup(None, "foo").as_deref(), Some("bar"));
    }

    #[test]
    fn mo_catalog_rejects_malformed_tables_and_plural_syntax() {
        assert!(Catalog::parse(b"ABCD").is_err());

        let bad_plural = mo_bytes(
            &[(
                b"",
                b"Content-Type: text/plain; charset=utf-8\nPlural-Forms: nplurals=2; plural=n @ 2;\n",
            )],
            false,
        );
        assert!(
            Catalog::parse(&bad_plural)
                .expect_err("bad plural expression should fail")
                .contains("invalid token")
        );
    }

    #[test]
    fn constrained_plural_parser_handles_c_style_subset() {
        let formula = Parser::new("n%10==1 && n%100!=11 ? 0 : 1")
            .expect("parser should tokenize")
            .parse()
            .expect("parser should parse");

        assert_eq!(formula.eval(1), Ok(0));
        assert_eq!(formula.eval(11), Ok(1));
        assert_eq!(formula.eval(22), Ok(1));
    }
}
