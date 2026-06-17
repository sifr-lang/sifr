// --- stdlib: sifr.textwrap ---
#[derive(Debug, Clone, PartialEq)]
struct TextWrapper {
    width: i64,
    initial_indent: String,
    subsequent_indent: String,
    expand_tabs: bool,
    tabsize: i64,
    replace_whitespace: bool,
    drop_whitespace: bool,
    break_on_hyphens: bool,
    fix_sentence_endings: bool,
    max_lines: Option<i64>,
    placeholder: String,
}
impl TextWrapper {
    fn new(
        width: i64,
        initial_indent: String,
        subsequent_indent: String,
        expand_tabs: bool,
        tabsize: i64,
        replace_whitespace: bool,
        drop_whitespace: bool,
        break_on_hyphens: bool,
        fix_sentence_endings: bool,
        max_lines: Option<i64>,
        placeholder: String,
    ) -> Self {
        let mut safe_tabsize: i64 = tabsize;
        if safe_tabsize <= (0 as i64) {
            safe_tabsize = 1 as i64;
        }
        return Self {
            width: width,
            initial_indent: format!("{}{}", initial_indent, "".to_string()),
            subsequent_indent: format!("{}{}", subsequent_indent, "".to_string()),
            expand_tabs: expand_tabs,
            tabsize: safe_tabsize,
            replace_whitespace: replace_whitespace,
            drop_whitespace: drop_whitespace,
            break_on_hyphens: break_on_hyphens,
            fix_sentence_endings: fix_sentence_endings,
            max_lines: max_lines,
            placeholder: format!("{}{}", placeholder, "".to_string()),
        };
    }
    fn wrap(&self, text: &String) -> Vec<String> {
        if self.width <= (0 as i64) {
            return vec![];
        }
        let prepared: String = _prepare_text(
            text,
            self.expand_tabs,
            self.tabsize,
            self.replace_whitespace,
        );
        let mut lines: Vec<String> = _wrap_with_indents(
            &prepared,
            self.width,
            &self.initial_indent.clone(),
            &self.subsequent_indent.clone(),
            self.break_on_hyphens,
            self.drop_whitespace,
        );
        if self.fix_sentence_endings {
            lines = _apply_sentence_endings_lines(&lines);
        }
        return _apply_max_lines(
            &lines,
            self.width,
            self.max_lines,
            &self.placeholder.clone(),
            self.drop_whitespace,
        );
    }
    fn fill(&self, text: &String) -> String {
        if self.width <= (0 as i64) {
            return "".to_string();
        }
        let lines: Vec<String> = self.wrap(text);
        let mut result: String = "".to_string();
        let mut i: i64 = 0 as i64;
        for line in lines.iter().cloned() {
            if i > (0 as i64) {
                result = format!("{}{}", result, "\n".to_string());
            }
            result = format!("{}{}", result, line);
            i = i + (1 as i64);
        }
        return result;
    }
}
fn _replace_whitespace_chars(text: &String, replace_tabs: bool) -> String {
    let normalized: String = text
        .replace(&"\n".to_string(), &" ".to_string())
        .replace(&"\r".to_string(), &" ".to_string())
        .replace(&"\u{b}".to_string(), &" ".to_string())
        .replace(&"\u{c}".to_string(), &" ".to_string());
    if replace_tabs {
        return normalized.replace(&"\t".to_string(), &" ".to_string());
    }
    return normalized;
}
fn _expand_tabs_impl(text: &String, tabsize: i64) -> String {
    let mut effective_tabsize: i64 = tabsize;
    if effective_tabsize <= (0 as i64) {
        effective_tabsize = 1 as i64;
    }
    let mut result: String = "".to_string();
    let mut column: i64 = 0 as i64;
    let mut i: i64 = 0 as i64;
    while i < (text.chars().count() as i64) {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = text.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "\t".to_string() {
                let mut spaces: i64 = effective_tabsize - (column % effective_tabsize);
                if spaces <= (0 as i64) {
                    spaces = effective_tabsize;
                }
                let mut j: i64 = 0 as i64;
                while j < spaces {
                    result = format!("{}{}", result, " ".to_string());
                    j = j + (1 as i64);
                }
                column = column + spaces;
            } else {
                if (ch == "\n".to_string()) || (ch == "\r".to_string()) {
                    result = format!("{}{}", result, ch);
                    column = 0 as i64;
                } else {
                    result = format!("{}{}", result, ch);
                    column = column + (1 as i64);
                }
            }
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _prepare_text(
    text: &String,
    expand_tabs: bool,
    tabsize: i64,
    replace_whitespace: bool,
) -> String {
    let mut prepared: String = format!("{}{}", text, "".to_string());
    if expand_tabs {
        prepared = _expand_tabs_impl(&prepared, tabsize);
    }
    if replace_whitespace {
        prepared = _replace_whitespace_chars(&prepared, true);
    }
    return prepared;
}
fn _split_word_units(word: &String, break_on_hyphens: bool) -> Vec<String> {
    if !break_on_hyphens {
        return vec![format!("{}{}", word, "".to_string())];
    }
    let parts: Vec<String> = word
        .split(&"-".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (parts.len() as i64) <= (1 as i64) {
        return vec![format!("{}{}", word, "".to_string())];
    }
    let mut units: Vec<String> = vec![];
    let mut index: i64 = 0 as i64;
    for part in parts.iter().cloned() {
        let is_last: bool = index == ((parts.len() as i64) - (1 as i64));
        if is_last {
            if (part.chars().count() as i64) > (0 as i64) {
                units.push(part);
            }
        } else {
            if (part.chars().count() as i64) == (0 as i64) {
                units.push("-".to_string());
            } else {
                units.push(format!("{}{}", part, "-".to_string()));
            }
        }
        index = index + (1 as i64);
    }
    if (units.len() as i64) == (0 as i64) {
        units.push(format!("{}{}", word, "".to_string()));
    }
    return units;
}
fn _trim_line(line: &String) -> String {
    let mut start: i64 = 0 as i64;
    while ((start < (line.chars().count() as i64))
        && (({
            let __sifr_index_str = &line;
            let __sifr_index_i = start;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        }) == Some(" ".to_string())))
    {
        start = start + (1 as i64);
    }
    let mut end: i64 = line.chars().count() as i64;
    while ((end > start)
        && (({
            let __sifr_index_str = &line;
            let __sifr_index_i = end - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        }) == Some(" ".to_string())))
    {
        end = end - (1 as i64);
    }
    return String::from_iter(
        (line)
            .chars()
            .skip((start).max(0) as usize)
            .take(((end).max(0) - (start).max(0)).max(0) as usize),
    );
}
fn _finalize_line(line: &String, drop_whitespace: bool) -> String {
    if drop_whitespace {
        return _trim_line(line);
    }
    return format!("{}{}", line, "".to_string());
}
fn _effective_content_width(total_width: i64, indent: &String) -> i64 {
    let available: i64 = total_width - (indent.chars().count() as i64);
    if available <= (0 as i64) {
        return 1 as i64;
    }
    return available;
}
fn _push_current_line(
    result: &mut Vec<String>,
    line: &String,
    indent: &String,
    drop_whitespace: bool,
) {
    let candidate: String = _finalize_line(
        &format!("{}{}", indent, line),
        drop_whitespace,
    );
    if drop_whitespace {
        if (candidate.chars().count() as i64) > (0 as i64) {
            result.push(candidate);
        }
    } else {
        result.push(candidate);
    }
}
fn _wrap_with_indents(
    text: &String,
    total_width: i64,
    initial_indent: &String,
    subsequent_indent: &String,
    break_on_hyphens: bool,
    drop_whitespace: bool,
) -> Vec<String> {
    let words: Vec<String> = text
        .split(&" ".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: Vec<String> = vec![];
    let mut current: String = "".to_string();
    let mut first_line: bool = true;
    let mut current_limit: i64 = _effective_content_width(total_width, initial_indent);
    for raw_word in words.iter().cloned() {
        let units: Vec<String> = _split_word_units(&raw_word, break_on_hyphens);
        for word in units.iter().cloned() {
            if (word.chars().count() as i64) == (0 as i64) {
                if drop_whitespace {
                    continue;
                }
                if (current.chars().count() as i64) > (0 as i64) {
                    if ((current.chars().count() as i64) + (1 as i64)) <= current_limit {
                        current = format!("{}{}", current, " ".to_string());
                    }
                }
                continue;
            }
            if (current.chars().count() as i64) == (0 as i64) {
                current = word;
            } else {
                if (((current.chars().count() as i64) + (1 as i64))
                    + (word.chars().count() as i64)) <= current_limit
                {
                    current = format!("{}{}{}", current, " ".to_string(), word);
                } else {
                    if first_line {
                        _push_current_line(
                            &mut result,
                            &current,
                            initial_indent,
                            drop_whitespace,
                        );
                        first_line = false;
                        current_limit = _effective_content_width(
                            total_width,
                            subsequent_indent,
                        );
                    } else {
                        _push_current_line(
                            &mut result,
                            &current,
                            subsequent_indent,
                            drop_whitespace,
                        );
                    }
                    current = word;
                }
            }
        }
    }
    if (current.chars().count() as i64) > (0 as i64) {
        if first_line {
            _push_current_line(&mut result, &current, initial_indent, drop_whitespace);
        } else {
            _push_current_line(
                &mut result,
                &current,
                subsequent_indent,
                drop_whitespace,
            );
        }
    }
    return result;
}
fn _apply_sentence_endings_line(text: &String) -> String {
    let mut result: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < (text.chars().count() as i64) {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = text.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            result = format!("{}{}", result, ch);
            if ((ch == ".".to_string()) || (ch == "!".to_string()))
                || (ch == "?".to_string())
            {
                let mut next_opt: Option<String> = None;
                if (i + (1 as i64)) < (text.chars().count() as i64) {
                    next_opt = {
                        let __sifr_index_str = &text;
                        let __sifr_index_i = i + (1 as i64);
                        let __sifr_index_norm = if __sifr_index_i < 0 {
                            ((__sifr_index_str.chars().count() as i64) + __sifr_index_i)
                                as usize
                        } else {
                            __sifr_index_i as usize
                        };
                        __sifr_index_str
                            .chars()
                            .nth(__sifr_index_norm)
                            .map(|c| c.to_string())
                    };
                }
                let mut next2_opt: Option<String> = None;
                if (i + (2 as i64)) < (text.chars().count() as i64) {
                    next2_opt = {
                        let __sifr_index_str = &text;
                        let __sifr_index_i = i + (2 as i64);
                        let __sifr_index_norm = if __sifr_index_i < 0 {
                            ((__sifr_index_str.chars().count() as i64) + __sifr_index_i)
                                as usize
                        } else {
                            __sifr_index_i as usize
                        };
                        __sifr_index_str
                            .chars()
                            .nth(__sifr_index_norm)
                            .map(|c| c.to_string())
                    };
                }
                if ((next_opt != None) && (next_opt == Some(" ".to_string()))) {
                    if ((next2_opt == None) || (next2_opt != Some(" ".to_string()))) {
                        result = format!("{}{}", result, " ".to_string());
                    }
                }
            }
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _apply_sentence_endings_lines(lines: &Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for line in lines.iter().cloned() {
        result.push(_apply_sentence_endings_line(&line));
    }
    return result;
}
fn _clone_lines(lines: &Vec<String>) -> Vec<String> {
    let mut copied: Vec<String> = vec![];
    for line in lines.iter().cloned() {
        copied.push(line);
    }
    return copied;
}
fn _apply_max_lines(
    lines: &Vec<String>,
    width: i64,
    max_lines: Option<i64>,
    placeholder: &String,
    drop_whitespace: bool,
) -> Vec<String> {
    let Some(max_lines) = max_lines else {
        return _clone_lines(lines);
    };
    let limit: i64 = max_lines;
    if limit <= (0 as i64) {
        return vec![];
    }
    if (lines.len() as i64) <= limit {
        return _clone_lines(lines);
    }
    let mut result: Vec<String> = vec![];
    let mut i: i64 = 0 as i64;
    while i < limit {
        let line_opt: Option<String> = {
            let __sifr_index_list = &lines;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(line_opt) = line_opt {
            result.push(line_opt);
        }
        i = i + (1 as i64);
    }
    if (result.len() as i64) == (0 as i64) {
        return result;
    }
    let mut effective_placeholder: String = format!("{}{}", placeholder, "".to_string());
    if width > (0 as i64) {
        if (effective_placeholder.chars().count() as i64) > width {
            effective_placeholder = String::from_iter(
                (effective_placeholder)
                    .chars()
                    .skip((0 as i64).max(0) as usize)
                    .take(((width).max(0) - (0 as i64).max(0)).max(0) as usize),
            );
        }
    }
    let last_index: i64 = (result.len() as i64) - (1 as i64);
    let last_opt: Option<String> = Some(result[last_index as usize].clone());
    if let Some(last_opt) = last_opt {
        let last: String = last_opt;
        let mut base: String = _trim_line(&last);
        let mut available: i64 = width - (effective_placeholder.chars().count() as i64);
        if available < (0 as i64) {
            available = 0 as i64;
        }
        if (base.chars().count() as i64) > available {
            base = _trim_line(
                &base
                    .chars()
                    .skip((0 as i64) as usize)
                    .take((available as usize) - ((0 as i64) as usize))
                    .collect::<String>(),
            );
        }
        if drop_whitespace {
            base = _trim_line(&base);
        }
        {
            let __idx_raw = last_index;
            let __idx_norm = if __idx_raw < 0 {
                (result.len() as i64) + __idx_raw
            } else {
                __idx_raw
            };
            if __idx_norm >= 0 {
                if let Some(__elem) = result.get_mut(__idx_norm as usize) {
                    *__elem = format!("{}{}", base, effective_placeholder);
                }
            }
        }
    }
    return result;
}

// --- stdlib: sifr.math ---
fn factorial(n: i64) -> i64 {
    if n < (0 as i64) {
        return 0 as i64;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 2 as i64;
    while i <= n {
        result = result * i;
        i = i + (1 as i64);
    }
    return result;
}
fn gcd(a: i64, b: i64) -> i64 {
    let mut x: i64 = a;
    let mut y: i64 = b;
    if x < (0 as i64) {
        x = (0 as i64) - x;
    }
    if y < (0 as i64) {
        y = (0 as i64) - y;
    }
    while y != (0 as i64) {
        let temp: i64 = y;
        y = x % y;
        x = temp;
    }
    return x;
}
fn lcm(a: i64, b: i64) -> i64 {
    if a == (0 as i64) {
        return 0 as i64;
    }
    if b == (0 as i64) {
        return 0 as i64;
    }
    let g: i64 = gcd(a, b);
    let mut x: i64 = a;
    if x < (0 as i64) {
        x = (0 as i64) - x;
    }
    let mut y: i64 = b;
    if y < (0 as i64) {
        y = (0 as i64) - y;
    }
    return (x / g) * y;
}
fn comb(n: i64, k: i64) -> i64 {
    if k < (0 as i64) {
        return 0 as i64;
    }
    if k > n {
        return 0 as i64;
    }
    if k == (0 as i64) {
        return 1 as i64;
    }
    if k == n {
        return 1 as i64;
    }
    let mut r: i64 = k;
    if r > (n - k) {
        r = n - k;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 0 as i64;
    while i < r {
        result = result * (n - i);
        result = result / (i + (1 as i64));
        i = i + (1 as i64);
    }
    return result;
}
fn perm(n: i64, k: i64) -> i64 {
    if k < (0 as i64) {
        return 0 as i64;
    }
    if k > n {
        return 0 as i64;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 0 as i64;
    while i < k {
        result = result * (n - i);
        i = i + (1 as i64);
    }
    return result;
}
fn log_base(x: f64, base: f64) -> f64 {
    return (x).ln() / (base).ln();
}
fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < (0.0 as f64) {
        return false;
    }
    if abs_tol < (0.0 as f64) {
        return false;
    }
    if a == b {
        return true;
    }
    if (((a).is_nan()) || ((b).is_nan())) {
        return false;
    }
    if (((a).is_infinite()) || ((b).is_infinite())) {
        return false;
    }
    let mut diff: f64 = a - b;
    if diff < (0.0 as f64) {
        diff = (0.0 as f64) - diff;
    }
    let mut a_abs: f64 = a;
    if a_abs < (0.0 as f64) {
        a_abs = (0.0 as f64) - a_abs;
    }
    let mut b_abs: f64 = b;
    if b_abs < (0.0 as f64) {
        b_abs = (0.0 as f64) - b_abs;
    }
    let mut rel_bound: f64 = rel_tol * (a_abs).max(b_abs);
    if abs_tol > rel_bound {
        rel_bound = abs_tol;
    }
    return diff <= rel_bound;
}
fn prod(data: &Vec<i64>) -> i64 {
    let mut result: i64 = 1 as i64;
    for val in data.iter().copied() {
        result = result * val;
    }
    return result;
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x == 0.0 {
            vec![__x, 0.0]
        } else {
            if !__x.is_finite() {
                vec![__x, 0.0]
            } else {
                {
                    let __bits: u64 = __x.to_bits();
                    let __sign_mask: u64 = (1 as u64) << 63;
                    let __frac_mask: u64 = ((1 as u64) << 52) - (1 as u64);
                    let __sign: u64 = __bits & __sign_mask;
                    let __exp: i32 = ((__bits >> 52) & (2047 as u64)) as i32;
                    let __frac: u64 = __bits & __frac_mask;
                    if __exp == 0 {
                        {
                            let __scaled: f64 = __x * (2.0 as f64).powi(54);
                            let __sbits: u64 = __scaled.to_bits();
                            let __sexp: i32 = ((__sbits >> 52) & (2047 as u64)) as i32;
                            let __sfrac: u64 = __sbits & __frac_mask;
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __sfrac,
                            );
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __frac,
                            );
                            let __e: i32 = __exp - 1022;
                            vec![__mant, __e as f64]
                        }
                    }
                }
            }
        }
    };
    let m: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(m) = m else {
        return f64::NAN;
    };
    return m;
}
fn frexp_exponent(x: f64) -> i64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x == 0.0 {
            vec![__x, 0.0]
        } else {
            if !__x.is_finite() {
                vec![__x, 0.0]
            } else {
                {
                    let __bits: u64 = __x.to_bits();
                    let __sign_mask: u64 = (1 as u64) << 63;
                    let __frac_mask: u64 = ((1 as u64) << 52) - (1 as u64);
                    let __sign: u64 = __bits & __sign_mask;
                    let __exp: i32 = ((__bits >> 52) & (2047 as u64)) as i32;
                    let __frac: u64 = __bits & __frac_mask;
                    if __exp == 0 {
                        {
                            let __scaled: f64 = __x * (2.0 as f64).powi(54);
                            let __sbits: u64 = __scaled.to_bits();
                            let __sexp: i32 = ((__sbits >> 52) & (2047 as u64)) as i32;
                            let __sfrac: u64 = __sbits & __frac_mask;
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __sfrac,
                            );
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __frac,
                            );
                            let __e: i32 = __exp - 1022;
                            vec![__mant, __e as f64]
                        }
                    }
                }
            }
        }
    };
    let exp_val: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(exp_val) = exp_val else {
        return 0 as i64;
    };
    return (exp_val).trunc() as i64;
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x.is_nan() {
            vec![f64::NAN, f64::NAN]
        } else {
            if __x.is_infinite() {
                vec![(0.0 as f64).copysign(__x), __x]
            } else {
                {
                    let __int = __x.trunc();
                    let mut __frac = __x - __int;
                    if __frac == 0.0 {
                        __frac = (0.0 as f64).copysign(__x);
                    }
                    vec![__frac, __int]
                }
            }
        }
    };
    let f: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(f) = f else {
        return f64::NAN;
    };
    return f;
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x.is_nan() {
            vec![f64::NAN, f64::NAN]
        } else {
            if __x.is_infinite() {
                vec![(0.0 as f64).copysign(__x), __x]
            } else {
                {
                    let __int = __x.trunc();
                    let mut __frac = __x - __int;
                    if __frac == 0.0 {
                        __frac = (0.0 as f64).copysign(__x);
                    }
                    vec![__frac, __int]
                }
            }
        }
    };
    let i: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(i) = i else {
        return f64::NAN;
    };
    return i;
}
fn pow(x: f64, y: f64) -> f64 {
    return (x).powf(y);
}

// --- stdlib: sifr.statistics ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StatisticsError {
    message: String,
}
impl StatisticsError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}
impl std::fmt::Display for StatisticsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}
impl std::error::Error for StatisticsError {}
fn median_grouped(data: &Vec<f64>, interval: f64) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0 as i64) {
        return Err(
            StatisticsError::new(
                "median_grouped requires at least one data point".to_string(),
            ),
        );
    }
    if interval <= (0.0 as f64) {
        return Err(
            StatisticsError::new("median_grouped: interval must be > 0".to_string()),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let mid_index: i64 = n / (2 as i64);
    let midpoint_opt: Option<f64> = {
        let __sifr_index_list = &sorted_data;
        let __sifr_index_i = mid_index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(midpoint_opt) = midpoint_opt else {
        return Err(StatisticsError::new("median_grouped: index error".to_string()));
    };
    let midpoint: f64 = midpoint_opt;
    let mut cf: i64 = 0 as i64;
    let mut f: i64 = 0 as i64;
    for value in sorted_data.iter().copied() {
        if value < midpoint {
            cf = cf + (1 as i64);
        } else {
            if value == midpoint {
                f = f + (1 as i64);
            }
        }
    }
    if f == (0 as i64) {
        return Err(
            StatisticsError::new("median_grouped: grouped frequency is zero".to_string()),
        );
    }
    let lower: f64 = midpoint - (interval / (2.0 as f64));
    return Ok(
        lower + (interval * ((((n as f64) / (2.0 as f64)) - (cf as f64)) / (f as f64))),
    );
}

// --- stdlib: sifr.html ---
fn escape(s: &String, quote: bool) -> String {
    let escaped: String = s
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&#x27;");
    if quote {
        return escaped;
    }
    return escaped
        .replace(&"&quot;".to_string(), &"\"".to_string())
        .replace(&"&#x27;".to_string(), &"\'".to_string());
}

#[derive(Debug, Clone)]
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for Error {
}

fn main() {
    let mut grouped: f64 = 0.0 as f64;
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let grouped_value: f64 = median_grouped(&vec![1.0 as f64, 2.0 as f64, 2.0 as f64, 3.0 as f64, 4.0 as f64], 1.0 as f64)?;
    grouped = grouped_value;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let _e = __sifr_try_err.clone();
        assert!(format!("{}", "median_grouped unexpected error".to_string()) == "rng_text_and_statistics_waiver_reduction_demo: pass".to_string());
    }
    assert!(grouped > (2.2 as f64));
    assert!(grouped < (2.3 as f64));
    let mut wrapper: TextWrapper = TextWrapper::new(12 as i64, "".to_string(), "".to_string(), true, 8 as i64, true, true, true, false, Some(2 as i64), "...".to_string());
    let wrapped: Vec<String> = wrapper.wrap(&"alpha beta gamma delta epsilon".to_string());
    assert!(format!("{:?}", wrapped) == "[\"alpha beta\", \"gamma del...\"]".to_string());
    let mut sentence_wrapper: TextWrapper = TextWrapper::new(40 as i64, "".to_string(), "".to_string(), true, 8 as i64, true, true, true, true, None, " [...]".to_string());
    let filled: String = sentence_wrapper.fill(&"Hello. World. Done!".to_string());
    assert!(filled == "Hello.  World.  Done!".to_string());
    let escaped: String = escape(&"<a \"x\">".to_string(), false);
    assert!(escaped == "&lt;a \"x\"&gt;".to_string());
    assert!(format!("{}", "rng_text_and_statistics_waiver_reduction_demo: pass".to_string()) == "rng_text_and_statistics_waiver_reduction_demo: pass".to_string());
}
