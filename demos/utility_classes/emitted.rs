// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        ParseError, SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec,
        SifrGeneratedStdlibSifrX2egraphlibX2eCycleError,
        SifrGeneratedStdlibSifrX2eipaddressX2eAddressValueError,
        SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address, SifrGeneratedStdlibSifrX2euuidX2eUUID,
        ValueError,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn sifr_generated_split_inline_option(token: &str) -> (bool, String, String) {
        let sifr_generated_chars_token: Vec<char> = token.chars().collect::<Vec<char>>();
        let mut key: String = String::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_token.len()) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_token.len());
                sifr_generated_chars_token
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if ch.is_some() && ch == Some("=".to_string()) {
                let mut value: String = String::new();
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                while &j < &SifrInt::from(sifr_generated_chars_token.len()) {
                    let part: Option<String> = {
                        let sifr_generated_string_index = j.clone();
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_token.len());
                        sifr_generated_chars_token
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    if let Some(part) = part {
                        value.push_str(part.as_str());
                    }
                    j = &j + &SifrInt::from_i64(1);
                }
                return (true, key, value);
            }
            if let Some(ch) = ch {
                key.push_str(ch.as_str());
            }
            i = &i + &SifrInt::from_i64(1);
        }
        (
            false,
            {
                let mut sifr_generated_concat: String = String::with_capacity(token.len());
                sifr_generated_concat.push_str(token);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            },
            String::new(),
        )
    }
    pub(crate) fn sifr_generated_is_digit_string(value: &str) -> bool {
        if value.is_empty() {
            return false;
        }
        for ch in value.chars().map(|c| c.to_string()) {
            if ch < "0".to_string() || ch > "9".to_string() {
                return false;
            }
        }
        true
    }
    pub(crate) fn sifr_generated_normalize_nargs(nargs: &str) -> String {
        if nargs.is_empty() {
            return "1".to_string();
        }
        if nargs == "?" || nargs == "*" || nargs == "+" {
            return {
                let mut sifr_generated_concat: String = String::with_capacity(nargs.len());
                sifr_generated_concat.push_str(nargs);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
        }
        if sifr_generated_is_digit_string(nargs) {
            let sifr_generated_try_res: Result<Option<String>, ParseError> = (|| {
                let parsed: SifrInt =
                    SifrInt::parse_decimal(&nargs, ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)
                        .map_err(|e| ParseError {
                            message: e.to_string(),
                        })?;
                if &parsed > &SifrInt::from_i64(0) {
                    return Ok(Some(parsed.to_string()));
                }
                Ok(None)
            })();
            match sifr_generated_try_res {
                Ok(Some(sifr_generated_ret_val)) => {
                    return sifr_generated_ret_val;
                }
                Ok(None) => {}
                Err(sifr_generated_try_err) => {
                    let _e = sifr_generated_try_err.clone();
                    return "1".to_string();
                }
            }
        }
        "1".to_string()
    }
    pub(crate) fn sifr_generated_nargs_is_multi(nargs: &str) -> bool {
        let normalized: String = sifr_generated_normalize_nargs(nargs);
        if normalized == "*" || normalized == "+" {
            return true;
        }
        if sifr_generated_is_digit_string(&normalized) {
            let sifr_generated_try_res: Result<bool, ParseError> = (|| {
                let parsed: SifrInt =
                    SifrInt::parse_decimal(&normalized, ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)
                        .map_err(|e| ParseError {
                            message: e.to_string(),
                        })?;
                Ok(&parsed > &SifrInt::from_i64(1))
            })();
            match sifr_generated_try_res {
                Ok(sifr_generated_ret_val) => {
                    return sifr_generated_ret_val;
                }
                Err(sifr_generated_try_err) => {
                    let _e = sifr_generated_try_err.clone();
                    return false;
                }
            }
        }
        false
    }
    pub(crate) fn sifr_generated_coerce_bool(raw: &str) -> Option<String> {
        let normalized: String = raw.to_lowercase();
        if normalized == "1" || normalized == "true" || normalized == "yes" || normalized == "on" {
            return Some("true".to_string());
        }
        if normalized == "0" || normalized == "false" || normalized == "no" || normalized == "off" {
            return Some("false".to_string());
        }
        None
    }
    pub(crate) fn sifr_generated_copy_token(value: &Option<String>) -> String {
        let Some(value) = value.as_ref() else {
            return String::new();
        };
        {
            let mut sifr_generated_concat: String = String::with_capacity(value.len());
            sifr_generated_concat.push_str(value);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_derive_dest(name: &str) -> String {
        let sifr_generated_chars_name: Vec<char> = name.chars().collect::<Vec<char>>();
        if name.starts_with("--") {
            return {
                let sifr_generated_slice_src = &sifr_generated_chars_name;
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start =
                    SifrInt::from_i64(2).clamp_slice_bound(sifr_generated_slice_len);
                let sifr_generated_slice_stop = SifrInt::from(sifr_generated_slice_src.len())
                    .clamp_slice_bound(sifr_generated_slice_len);
                sifr_generated_slice_src
                    .iter()
                    .skip(sifr_generated_slice_start)
                    .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                    .copied()
                    .collect::<String>()
            }
            .replace('-', "_");
        }
        if name.starts_with("-") {
            return {
                let sifr_generated_slice_src = &sifr_generated_chars_name;
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start =
                    SifrInt::from_i64(1).clamp_slice_bound(sifr_generated_slice_len);
                let sifr_generated_slice_stop = SifrInt::from(sifr_generated_slice_src.len())
                    .clamp_slice_bound(sifr_generated_slice_len);
                sifr_generated_slice_src
                    .iter()
                    .skip(sifr_generated_slice_start)
                    .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                    .copied()
                    .collect::<String>()
            }
            .replace('-', "_");
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(name.len());
            sifr_generated_concat.push_str(name);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_is_option_like_token(
        specs: &[SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec],
        token: &str,
    ) -> bool {
        if token == "--" {
            return true;
        }
        if token.starts_with("--") {
            return true;
        }
        let (inline_has_value, inline_name, inline_value) =
            sifr_generated_split_inline_option(token);
        let _chars_inline_name: Vec<char> = inline_name.chars().collect::<Vec<char>>();
        let _chars_inline_value: Vec<char> = inline_value.chars().collect::<Vec<char>>();
        let _ = inline_value;
        let lookup_name: String = if inline_has_value {
            {
                let mut sifr_generated_concat: String = String::with_capacity(inline_name.len());
                sifr_generated_concat.push_str(inline_name.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
        } else {
            let mut sifr_generated_concat: String = String::with_capacity(token.len());
            sifr_generated_concat.push_str(token);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        for spec in specs.iter().cloned() {
            if spec.kind.clone() == "positional" {
                continue;
            }
            if spec.name.clone() == lookup_name {
                return true;
            }
        }
        false
    }
    pub(crate) fn sifr_generated_contains_int(values: &[SifrInt], target: SifrInt) -> bool {
        for value in values.iter().cloned() {
            if &value == &target {
                return true;
            }
        }
        false
    }
    pub(crate) fn topological_sort(
        num_nodes: SifrInt,
        from_nodes: &[SifrInt],
        to_nodes: &[SifrInt],
    ) -> Result<Vec<SifrInt>, SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> {
        let mut result: Vec<SifrInt> = Vec::new();
        let mut visited: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &num_nodes {
            visited.push(SifrInt::from_i64(0));
            i = &i + &SifrInt::from_i64(1);
        }
        let mut processed: SifrInt = SifrInt::from_i64(0);
        while &processed < &num_nodes {
            let mut found_any: bool = false;
            let mut node: SifrInt = SifrInt::from_i64(0);
            while &SifrInt::from_i64(0) <= &node && &node < &SifrInt::from(visited.len()) {
                let v: Option<SifrInt> = {
                    let sifr_generated_checked_read_collection = &visited;
                    let sifr_generated_checked_read_index = node.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(v) = v.clone()
                    && &v == &SifrInt::from_i64(0)
                {
                    let mut has_dep: bool = false;
                    let mut j: SifrInt = SifrInt::from_i64(0);
                    while &j < &SifrInt::from(to_nodes.len()) {
                        let to_val: Option<SifrInt> = {
                            let sifr_generated_checked_read_collection = &to_nodes;
                            let sifr_generated_checked_read_index = j.clone();
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        let from_val: Option<SifrInt> = {
                            let sifr_generated_checked_read_collection = &from_nodes;
                            let sifr_generated_checked_read_index = j.clone();
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        if let Some(to_val) = to_val.clone()
                            && let Some(from_val) = from_val.clone()
                            && &to_val == &node
                        {
                            let dep_v: Option<SifrInt> = {
                                let sifr_generated_checked_read_collection = &visited;
                                let sifr_generated_checked_read_index = from_val.clone();
                                let sifr_generated_checked_read_normalized =
                                    sifr_generated_checked_read_index.normalize_index_or_len(
                                        sifr_generated_checked_read_collection.len(),
                                    );
                                sifr_generated_checked_read_collection
                                    .get(sifr_generated_checked_read_normalized)
                                    .cloned()
                            };
                            if let Some(dep_v) = dep_v.clone()
                                && &dep_v == &SifrInt::from_i64(0)
                            {
                                has_dep = true;
                            }
                        }
                        j = &j + &SifrInt::from_i64(1);
                    }
                    if !has_dep {
                        result.push(node.clone());
                        {
                            let sifr_generated_assign_value = SifrInt::from_i64(1);
                            {
                                let sifr_generated_index_raw = node.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(visited.len());
                                if let Some(sifr_generated_elem) =
                                    visited.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                        processed = &processed + &SifrInt::from_i64(1);
                        found_any = true;
                    }
                }
                node = &node + &SifrInt::from_i64(1);
            }
            if !found_any {
                return Err(SifrGeneratedStdlibSifrX2egraphlibX2eCycleError::new(
                    "cycle detected in graph".to_string(),
                ));
            }
        }
        Ok(result)
    }
    pub(crate) fn is_valid_ipv4(addr: &str) -> bool {
        let parts: Vec<String> = addr
            .split('.')
            .map(::std::string::ToString::to_string)
            .collect::<Vec<String>>();
        if &SifrInt::from(parts.len()) != &SifrInt::from_i64(4) {
            return false;
        }
        for part in parts.iter().cloned() {
            let sifr_generated_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
            if &SifrInt::from(sifr_generated_chars_part.len()) == &SifrInt::from_i64(0) {
                return false;
            }
            if &SifrInt::from(sifr_generated_chars_part.len()) > &SifrInt::from_i64(3) {
                return false;
            }
            if &SifrInt::from(sifr_generated_chars_part.len()) > &SifrInt::from_i64(1) {
                let first_digit: Option<String> = {
                    let sifr_generated_string_index = SifrInt::from_i64(0);
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_chars_part.len());
                    sifr_generated_chars_part
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(|character| character.to_string());
                if first_digit.is_some() && first_digit == Some("0".to_string()) {
                    return false;
                }
            }
            let val: SifrInt = sifr_generated_parse_int(&part);
            if &val < &SifrInt::from_i64(0) {
                return false;
            }
            if &val > &SifrInt::from_i64(255) {
                return false;
            }
        }
        true
    }
    pub(crate) fn sifr_generated_parse_int(s: &str) -> SifrInt {
        let sifr_generated_chars_s: Vec<char> = s.chars().collect::<Vec<char>>();
        let mut result: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_s.len()) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_s.len());
                sifr_generated_chars_s
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch {
                if ch == "0" {
                    result = &result * &SifrInt::from_i64(10);
                } else if ch == "1" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(1);
                } else if ch == "2" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(2);
                } else if ch == "3" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(3);
                } else if ch == "4" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(4);
                } else if ch == "5" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(5);
                } else if ch == "6" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(6);
                } else if ch == "7" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(7);
                } else if ch == "8" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(8);
                } else if ch == "9" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(9);
                } else {
                    return -&SifrInt::from_i64(1);
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        result.clone()
    }
    pub(crate) fn sifr_generated_ip_to_int_raw(addr: &str) -> SifrInt {
        let parts: Vec<String> = addr
            .split('.')
            .map(::std::string::ToString::to_string)
            .collect::<Vec<String>>();
        let mut result: SifrInt = SifrInt::from_i64(0);
        for part in parts.iter().cloned() {
            let val: SifrInt = sifr_generated_parse_int(&part);
            result = &(&result * &SifrInt::from_i64(256)) + &val;
        }
        result.clone()
    }
    pub(crate) fn sifr_generated_in_ipv4_range(
        value: SifrInt,
        start: SifrInt,
        end: SifrInt,
    ) -> bool {
        if &value < &start {
            return false;
        }
        if &value > &end {
            return false;
        }
        true
    }
    pub(crate) fn sifr_generated_is_private_ipv4_value(value: SifrInt) -> bool {
        let private_hit: bool = if sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(0),
            SifrInt::from_i64(16_777_215),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(167_772_160),
            SifrInt::from_i64(184_549_375),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(2_130_706_432),
            SifrInt::from_i64(2_147_483_647),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(2_851_995_648),
            SifrInt::from_i64(2_852_061_183),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(2_886_729_728),
            SifrInt::from_i64(2_887_778_303),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(3_221_225_472),
            SifrInt::from_i64(3_221_225_727),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(3_221_225_642),
            SifrInt::from_i64(3_221_225_643),
        )
            || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(3_221_225_984),
                SifrInt::from_i64(3_221_226_239),
            ) || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(3_232_235_520),
                SifrInt::from_i64(3_232_301_055),
            ) || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(3_323_068_416),
                SifrInt::from_i64(3_323_199_487),
            ) || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(3_325_256_704),
                SifrInt::from_i64(3_325_256_959),
            ) || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(3_405_803_776),
                SifrInt::from_i64(3_405_804_031),
            ) || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(4_026_531_840),
                SifrInt::from_i64(4_294_967_295),
            ) || &value
                == &SifrInt::from_i64(4_294_967_295)))))))))))))
        {
            true
        } else {
            false
        };
        if private_hit {
            if &value == &SifrInt::from_i64(3_221_225_481) {
                return false;
            }
            if &value == &SifrInt::from_i64(3_221_225_482) {
                return false;
            }
        }
        private_hit
    }
    pub(crate) fn int_to_ip(val: SifrInt) -> String {
        if &val < &SifrInt::from_i64(0) || &val > &SifrInt::from_i64(4_294_967_295) {
            return "0.0.0.0".to_string();
        }
        let a: SifrInt = val.floor_div_known_nonzero(&SifrInt::from_i64(16_777_216));
        let mut rem: SifrInt = val.floor_mod_known_nonzero(&SifrInt::from_i64(16_777_216));
        let b: SifrInt = rem.floor_div_known_nonzero(&SifrInt::from_i64(65536));
        rem = rem.floor_mod_known_nonzero(&SifrInt::from_i64(65536));
        let c: SifrInt = rem.floor_div_known_nonzero(&SifrInt::from_i64(256));
        let d: SifrInt = rem.floor_mod_known_nonzero(&SifrInt::from_i64(256));
        {
            let mut sifr_generated_concat: String = String::with_capacity(1usize + 1usize + 1usize);
            sifr_generated_concat.push_str(a.to_string().as_str());
            sifr_generated_concat.push('.');
            sifr_generated_concat.push_str(b.to_string().as_str());
            sifr_generated_concat.push('.');
            sifr_generated_concat.push_str(c.to_string().as_str());
            sifr_generated_concat.push('.');
            sifr_generated_concat.push_str(d.to_string().as_str());
            sifr_generated_concat
        }
    }
    pub(crate) fn is_global(addr: &str) -> bool {
        if !is_valid_ipv4(addr) {
            return false;
        }
        let val: SifrInt = sifr_generated_ip_to_int_raw(addr);
        if sifr_generated_in_ipv4_range(
            val.clone(),
            SifrInt::from_i64(1_681_915_904),
            SifrInt::from_i64(1_686_110_207),
        ) {
            return false;
        }
        !sifr_generated_is_private_ipv4_value(val.clone())
    }
    pub(crate) fn is_link_local(addr: &str) -> bool {
        if !is_valid_ipv4(addr) {
            return false;
        }
        let val: SifrInt = sifr_generated_ip_to_int_raw(addr);
        sifr_generated_in_ipv4_range(
            val.clone(),
            SifrInt::from_i64(2_851_995_648),
            SifrInt::from_i64(2_852_061_183),
        )
    }
    pub(crate) fn ip_address(
        addr: &str,
    ) -> Result<
        SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address,
        SifrGeneratedStdlibSifrX2eipaddressX2eAddressValueError,
    > {
        if !is_valid_ipv4(addr) {
            return Err(
                SifrGeneratedStdlibSifrX2eipaddressX2eAddressValueError::new(
                    "invalid IPv4 address".to_string(),
                ),
            );
        }
        Ok(SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address::new(
            addr.to_owned(),
        ))
    }
    pub(crate) fn uuid4() -> String {
        ::sifr_stdlib::uuid::uuid4()
    }
    pub(crate) fn sifr_generated_to_lower_hex_char(ch: &str) -> String {
        if ch == "A" {
            return "a".to_string();
        }
        if ch == "B" {
            return "b".to_string();
        }
        if ch == "C" {
            return "c".to_string();
        }
        if ch == "D" {
            return "d".to_string();
        }
        if ch == "E" {
            return "e".to_string();
        }
        if ch == "F" {
            return "f".to_string();
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(ch.len());
            sifr_generated_concat.push_str(ch);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_is_hex_char(ch: &str) -> bool {
        if ch == "0" {
            return true;
        }
        if ch == "1" {
            return true;
        }
        if ch == "2" {
            return true;
        }
        if ch == "3" {
            return true;
        }
        if ch == "4" {
            return true;
        }
        if ch == "5" {
            return true;
        }
        if ch == "6" {
            return true;
        }
        if ch == "7" {
            return true;
        }
        if ch == "8" {
            return true;
        }
        if ch == "9" {
            return true;
        }
        if ch == "a" {
            return true;
        }
        if ch == "b" {
            return true;
        }
        if ch == "c" {
            return true;
        }
        if ch == "d" {
            return true;
        }
        if ch == "e" {
            return true;
        }
        if ch == "f" {
            return true;
        }
        if ch == "A" {
            return true;
        }
        if ch == "B" {
            return true;
        }
        if ch == "C" {
            return true;
        }
        if ch == "D" {
            return true;
        }
        if ch == "E" {
            return true;
        }
        if ch == "F" {
            return true;
        }
        false
    }
    pub(crate) fn sifr_generated_hex_digit_value(ch: &str) -> SifrInt {
        if ch == "0" {
            return SifrInt::from_i64(0);
        }
        if ch == "1" {
            return SifrInt::from_i64(1);
        }
        if ch == "2" {
            return SifrInt::from_i64(2);
        }
        if ch == "3" {
            return SifrInt::from_i64(3);
        }
        if ch == "4" {
            return SifrInt::from_i64(4);
        }
        if ch == "5" {
            return SifrInt::from_i64(5);
        }
        if ch == "6" {
            return SifrInt::from_i64(6);
        }
        if ch == "7" {
            return SifrInt::from_i64(7);
        }
        if ch == "8" {
            return SifrInt::from_i64(8);
        }
        if ch == "9" {
            return SifrInt::from_i64(9);
        }
        if ch == "a" || ch == "A" {
            return SifrInt::from_i64(10);
        }
        if ch == "b" || ch == "B" {
            return SifrInt::from_i64(11);
        }
        if ch == "c" || ch == "C" {
            return SifrInt::from_i64(12);
        }
        if ch == "d" || ch == "D" {
            return SifrInt::from_i64(13);
        }
        if ch == "e" || ch == "E" {
            return SifrInt::from_i64(14);
        }
        if ch == "f" || ch == "F" {
            return SifrInt::from_i64(15);
        }
        -&SifrInt::from_i64(1)
    }
    pub(crate) fn sifr_generated_substring(value: &str, start: SifrInt, end: SifrInt) -> String {
        let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let mut result: String = String::new();
        let mut i: SifrInt = start.clone();
        while &i < &end {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_value.len());
                sifr_generated_chars_value
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch {
                result.push_str(ch.as_str());
            }
            i = &i + &SifrInt::from_i64(1);
        }
        result
    }
    pub(crate) fn sifr_generated_starts_with(value: &str, prefix: &str) -> bool {
        let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let sifr_generated_chars_prefix: Vec<char> = prefix.chars().collect::<Vec<char>>();
        if &SifrInt::from(sifr_generated_chars_value.len())
            < &SifrInt::from(sifr_generated_chars_prefix.len())
        {
            return false;
        }
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_prefix.len()) {
            let left: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_value.len());
                sifr_generated_chars_value
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let right: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_prefix.len());
                sifr_generated_chars_prefix
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if left != right {
                return false;
            }
            i = &i + &SifrInt::from_i64(1);
        }
        true
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn sifr_generated_canonical_uuid_text(
        input_text: &str,
    ) -> Result<String, ValueError> {
        let mut normalized_input: String = {
            let mut sifr_generated_concat: String = String::with_capacity(input_text.len());
            sifr_generated_concat.push_str(input_text);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        let mut sifr_generated_chars_normalized_input: Vec<char> =
            if sifr_generated_starts_with(&normalized_input, &"urn:uuid:".to_string()) {
                {
                    normalized_input = sifr_generated_substring(
                        &normalized_input,
                        SifrInt::from_i64(9),
                        SifrInt::from(normalized_input.chars().count()),
                    );
                    normalized_input.chars().collect::<Vec<char>>()
                }
            } else {
                normalized_input.chars().collect::<Vec<char>>()
            };
        if &SifrInt::from(sifr_generated_chars_normalized_input.len()) >= &SifrInt::from_i64(2) {
            let first: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(0);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let last: Option<String> = {
                let sifr_generated_string_index =
                    &SifrInt::from(normalized_input.chars().count()) - &SifrInt::from_i64(1);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if first == Some("{".to_string()) && last == Some("}".to_string()) {
                normalized_input = sifr_generated_substring(
                    &normalized_input,
                    SifrInt::from_i64(1),
                    &SifrInt::from(normalized_input.chars().count()) - &SifrInt::from_i64(1),
                );
                sifr_generated_chars_normalized_input =
                    normalized_input.chars().collect::<Vec<char>>();
            }
        }
        let input_len: SifrInt = SifrInt::from(sifr_generated_chars_normalized_input.len());
        let mut hex_only: String = String::new();
        let mut sifr_generated_chars_hex_only: Vec<char> = hex_only.chars().collect::<Vec<char>>();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &input_len {
            let ch_opt: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch_opt) = ch_opt {
                let ch: String = ch_opt;
                if ch == "-" {
                } else {
                    if !sifr_generated_is_hex_char(&ch) {
                        return Err(ValueError::new("invalid UUID hex string".to_string()));
                    }
                    let sifr_generated_string_concat_hex_only_0 =
                        sifr_generated_to_lower_hex_char(&ch);
                    hex_only.push_str(sifr_generated_string_concat_hex_only_0.as_str());
                    sifr_generated_chars_hex_only
                        .extend(sifr_generated_string_concat_hex_only_0.as_str().chars());
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        if &SifrInt::from(sifr_generated_chars_hex_only.len()) != &SifrInt::from_i64(32) {
            return Err(ValueError::new(
                "UUID hex string must be 32 hex characters".to_string(),
            ));
        }
        if &input_len == &SifrInt::from_i64(36) {
            let h1: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(8);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let h2: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(13);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let h3: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(18);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let h4: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(23);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if h1 != Some("-".to_string())
                || h2 != Some("-".to_string())
                || h3 != Some("-".to_string())
                || h4 != Some("-".to_string())
            {
                return Err(ValueError::new("invalid UUID hex string".to_string()));
            }
        } else if &input_len != &SifrInt::from_i64(32) {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
        let mut canonical: String = String::new();
        let mut j: SifrInt = SifrInt::from_i64(0);
        while &j < &SifrInt::from(sifr_generated_chars_hex_only.len()) {
            if &j == &SifrInt::from_i64(8)
                || &j == &SifrInt::from_i64(12)
                || &j == &SifrInt::from_i64(16)
                || &j == &SifrInt::from_i64(20)
            {
                canonical.push('-');
            }
            let part: Option<String> = {
                let sifr_generated_string_index = j.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_hex_only.len());
                sifr_generated_chars_hex_only
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(part) = part {
                canonical.push_str(part.as_str());
            }
            j = &j + &SifrInt::from_i64(1);
        }
        Ok(canonical)
    }
    pub(crate) fn uuid4_obj() -> SifrGeneratedStdlibSifrX2euuidX2eUUID {
        SifrGeneratedStdlibSifrX2euuidX2eUUID::new(uuid4())
    }
    pub(crate) fn uuid_from_hex(
        hex_str: &str,
    ) -> Result<SifrGeneratedStdlibSifrX2euuidX2eUUID, ValueError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2euuidX2eUUID, ValueError>,
            ValueError,
        > = (|| {
            let canonical: String = sifr_generated_canonical_uuid_text(hex_str)?;
            Ok(Ok(SifrGeneratedStdlibSifrX2euuidX2eUUID::new(canonical)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(ValueError::new(e.message.clone()))
        })
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec {
        pub name: String,
        pub dest: String,
        pub kind: String,
        pub default_value: String,
        pub nargs: String,
        pub type_name: String,
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec {
        #[must_use]
        pub fn new(
            name: String,
            dest: String,
            kind: String,
            default_value: String,
            nargs: String,
            type_name: String,
        ) -> Self {
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = {
                let mut sifr_generated_concat: String = String::with_capacity(name.len());
                sifr_generated_concat.push_str(name.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_a5eb0667427cce95_64657374: String = {
                let mut sifr_generated_concat: String = String::with_capacity(dest.len());
                sifr_generated_concat.push_str(dest.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_ef9c96d721673243_6b696e64: String = {
                let mut sifr_generated_concat: String = String::with_capacity(kind.len());
                sifr_generated_concat.push_str(kind.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_c029ceb935ca1970_64656661756c745f76616c7565: String = {
                let mut sifr_generated_concat: String = String::with_capacity(default_value.len());
                sifr_generated_concat.push_str(default_value.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_c4fccdff6d365b00_6e61726773: String =
                sifr_generated_normalize_nargs(&nargs);
            let sifr_generated_field_value_c23e4d7df5c6ddd5_747970655f6e616d65: String = {
                let mut sifr_generated_concat: String = String::with_capacity(type_name.len());
                sifr_generated_concat.push_str(type_name.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            Self {
                name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
                dest: sifr_generated_field_value_a5eb0667427cce95_64657374,
                kind: sifr_generated_field_value_ef9c96d721673243_6b696e64,
                default_value:
                    sifr_generated_field_value_c029ceb935ca1970_64656661756c745f76616c7565,
                nargs: sifr_generated_field_value_c4fccdff6d365b00_6e61726773,
                type_name: sifr_generated_field_value_c23e4d7df5c6ddd5_747970655f6e616d65,
            }
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "ArgumentSpec(name={}, dest={}, kind={}, default_value={}, nargs={}, type_name={})",
                self.name, self.dest, self.kind, self.default_value, self.nargs, self.type_name
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        pub str_values: Vec<(String, String)>,
        pub bool_values: Vec<(String, bool)>,
        pub list_values: Vec<(String, Vec<String>)>,
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        #[must_use]
        pub const fn new() -> Self {
            let sifr_generated_field_value_ad3f1317446e6bf0_5f7374725f76616c756573: Vec<(
                String,
                String,
            )> = Vec::new();
            let sifr_generated_field_value_1179342d80643edd_5f626f6f6c5f76616c756573: Vec<(
                String,
                bool,
            )> = Vec::new();
            let sifr_generated_field_value_9f4a2d21db1be045_5f6c6973745f76616c756573: Vec<(
                String,
                Vec<String>,
            )> = Vec::new();
            Self {
                str_values: sifr_generated_field_value_ad3f1317446e6bf0_5f7374725f76616c756573,
                bool_values: sifr_generated_field_value_1179342d80643edd_5f626f6f6c5f76616c756573,
                list_values: sifr_generated_field_value_9f4a2d21db1be045_5f6c6973745f76616c756573,
            }
        }
    }
    impl ::std::default::Default for SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        fn default() -> Self {
            Self::new()
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        pub fn set(&mut self, name: &str, value: &str) {
            let mut updated: Vec<(String, String)> = Vec::new();
            let mut replaced: bool = false;
            for (key, current) in self.str_values.iter().cloned() {
                if key == *name {
                    updated.push((name.to_string(), value.to_string()));
                    replaced = true;
                } else {
                    updated.push((key.to_owned(), current.to_owned()));
                }
            }
            if !replaced {
                updated.push((name.to_string(), value.to_string()));
            }
            self.str_values = updated;
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        pub fn set_bool(&mut self, name: &str, value: bool) {
            let mut updated: Vec<(String, bool)> = Vec::new();
            let mut replaced: bool = false;
            for (key, current) in self.bool_values.iter().cloned() {
                if key == *name {
                    updated.push((name.to_string(), value));
                    replaced = true;
                } else {
                    updated.push((key.to_owned(), current));
                }
            }
            if !replaced {
                updated.push((name.to_string(), value));
            }
            self.bool_values = updated;
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        pub fn set_list(&mut self, name: &str, values: &[String]) {
            let mut copied: Vec<String> = Vec::new();
            for value in values.iter().cloned() {
                copied.push(value.to_string());
            }
            let mut updated: Vec<(String, Vec<String>)> = Vec::new();
            for (key, current) in self.list_values.iter().cloned() {
                if key == *name {
                    continue;
                }
                updated.push((key.to_owned(), current.to_vec()));
            }
            updated.push((name.to_string(), copied.to_vec()));
            self.list_values = updated;
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        #[must_use]
        pub fn get(&self, name: &str, default: &str) -> String {
            for (key, value) in self.str_values.iter().cloned() {
                if key == *name {
                    return {
                        let mut sifr_generated_concat: String = String::with_capacity(value.len());
                        sifr_generated_concat.push_str(value.as_str());
                        sifr_generated_concat.push_str("");
                        sifr_generated_concat
                    };
                }
            }
            {
                let mut sifr_generated_concat: String = String::with_capacity(default.len());
                sifr_generated_concat.push_str(default);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        #[must_use]
        pub fn get_bool(&self, name: &str, default: bool) -> bool {
            for (key, value) in self.bool_values.iter().cloned() {
                if key == *name {
                    return value;
                }
            }
            for (key2, value2) in self.str_values.iter().cloned() {
                if key2 != *name {
                    continue;
                }
                let normalized: String = value2.to_lowercase();
                if normalized == "1"
                    || normalized == "true"
                    || normalized == "yes"
                    || normalized == "on"
                {
                    return true;
                }
                if normalized == "0"
                    || normalized == "false"
                    || normalized == "no"
                    || normalized == "off"
                {
                    return false;
                }
            }
            default
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        pub fn merge_from(&mut self, other: &SifrGeneratedStdlibSifrX2eargparseX2eNamespace) {
            for (key, value) in other.str_values.iter().cloned() {
                self.set(&key, &value);
            }
            for (key2, value2) in other.bool_values.iter().cloned() {
                self.set_bool(&key2, value2);
            }
            for (key3, values3) in other.list_values.iter().cloned() {
                self.set_list(&key3, &values3);
            }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        pub prog: String,
        pub specs: Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec>,
        pub subparsers_dest: String,
        pub subparsers: Vec<(
            String,
            Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec>,
        )>,
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        pub fn new(prog: String) -> Self {
            let sifr_generated_field_value_68bfad6e66c74136_5f70726f67: String = {
                let mut sifr_generated_concat: String = String::with_capacity(prog.len());
                sifr_generated_concat.push_str(prog.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_fe08c9a04e4710ae_5f7370656373: Vec<
                SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec,
            > = Vec::new();
            let sifr_generated_field_value_d0dd847dfcb3acd5_5f737562706172736572735f64657374: String = "command"
                .to_string();
            let sifr_generated_field_value_bca9a861c9b63fd8_5f73756270617273657273: Vec<(
                String,
                Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec>,
            )> = Vec::new();
            Self {
                prog: sifr_generated_field_value_68bfad6e66c74136_5f70726f67,
                specs: sifr_generated_field_value_fe08c9a04e4710ae_5f7370656373,
                subparsers_dest:
                    sifr_generated_field_value_d0dd847dfcb3acd5_5f737562706172736572735f64657374,
                subparsers: sifr_generated_field_value_bca9a861c9b63fd8_5f73756270617273657273,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        pub fn sifr_generated_append_spec(
            &mut self,
            name: &str,
            dest: &str,
            action: &str,
            default: &str,
            nargs: &str,
            type_name: &str,
        ) {
            let mut resolved_dest: String = {
                let mut sifr_generated_concat: String = String::with_capacity(dest.len());
                sifr_generated_concat.push_str(dest);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            if resolved_dest.is_empty() {
                resolved_dest = sifr_generated_derive_dest(name);
            }
            let mut kind: String = "positional".to_string();
            if name.starts_with("-") {
                if action == "store_true" {
                    kind = "flag".to_string();
                } else {
                    kind = "option".to_string();
                }
            }
            let spec: SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec =
                SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec::new(
                    name.to_owned(),
                    resolved_dest,
                    kind,
                    default.to_owned(),
                    nargs.to_owned(),
                    type_name.to_owned(),
                );
            self.specs.push(spec.clone());
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        pub fn add_argument(&mut self, name: &str, dest: &str, action: &str, default: &str) {
            self.add_argument_typed(
                name,
                dest,
                action,
                default,
                &"1".to_string(),
                &"str".to_string(),
            );
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        pub fn add_argument_typed(
            &mut self,
            name: &str,
            dest: &str,
            action: &str,
            default: &str,
            nargs: &str,
            type_name: &str,
        ) {
            let mut normalized_type: String = {
                let mut sifr_generated_concat: String = String::with_capacity(type_name.len());
                sifr_generated_concat.push_str(type_name);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            if normalized_type != "int"
                && normalized_type != "float"
                && normalized_type != "bool"
                && normalized_type != "str"
            {
                normalized_type = "str".to_string();
            }
            self.sifr_generated_append_spec(name, dest, action, default, nargs, &normalized_type);
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        pub fn sifr_generated_find_subparser(
            &self,
            name: &str,
        ) -> Option<Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec>> {
            for (parser_name, parser_specs) in self.subparsers.iter().cloned() {
                if parser_name == *name {
                    return Some(parser_specs);
                }
            }
            None
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        pub fn sifr_generated_coerce_token(
            &self,
            spec: &SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec,
            token: &str,
        ) -> Option<String> {
            if spec.type_name.clone() == "int" {
                let sifr_generated_try_res: Result<Option<String>, ParseError> = (|| {
                    let parsed_int: SifrInt =
                        SifrInt::parse_decimal(&token, ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)
                            .map_err(|e| ParseError {
                                message: e.to_string(),
                            })?;
                    Ok(Some(parsed_int.to_string()))
                })(
                );
                match sifr_generated_try_res {
                    Ok(sifr_generated_ret_val) => {
                        return sifr_generated_ret_val;
                    }
                    Err(sifr_generated_try_err) => {
                        let _e = sifr_generated_try_err.clone();
                        return None;
                    }
                }
            }
            if spec.type_name.clone() == "float" {
                let sifr_generated_try_res: Result<Option<String>, ParseError> = (|| {
                    let parsed_float: f64 = token.parse::<f64>().map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                    Ok(Some(parsed_float.to_string()))
                })(
                );
                match sifr_generated_try_res {
                    Ok(sifr_generated_ret_val) => {
                        return sifr_generated_ret_val;
                    }
                    Err(sifr_generated_try_err) => {
                        let _e = sifr_generated_try_err.clone();
                        return None;
                    }
                }
            }
            if spec.type_name.clone() == "bool" {
                return sifr_generated_coerce_bool(token);
            }
            Some({
                let mut sifr_generated_concat: String = String::with_capacity(token.len());
                sifr_generated_concat.push_str(token);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            })
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        #[expect(
            clippy::too_many_lines,
            reason = "one generated Rust function preserves one typed Sifr function"
        )]
        pub fn sifr_generated_collect_option_values(
            &self,
            args: &[String],
            start: &SifrInt,
            spec: &SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec,
            force_positional: bool,
        ) -> (Vec<String>, SifrInt) {
            let mut values: Vec<String> = Vec::new();
            let mut i: SifrInt = start.clone();
            if spec.nargs.clone() == "?" {
                if &i >= &SifrInt::from(args.len()) {
                    return (values.to_vec(), i.clone());
                }
                let token_opt: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let Some(token_opt_value_6bfb1ab9e84751f4) = token_opt else {
                    return (values.to_vec(), &i + &SifrInt::from_i64(1));
                };
                let token_one_value_6bbe1fb9e813ac55: String =
                    sifr_generated_copy_token(&Some(token_opt_value_6bfb1ab9e84751f4.to_owned()));
                if !force_positional
                    && sifr_generated_is_option_like_token(
                        &self.specs,
                        &token_one_value_6bbe1fb9e813ac55,
                    )
                {
                    return (values.to_vec(), i.clone());
                }
                values.push(token_one_value_6bbe1fb9e813ac55.to_owned());
                return (values.to_vec(), &i + &SifrInt::from_i64(1));
            }
            if spec.nargs.clone() == "*" || spec.nargs.clone() == "+" {
                while &i < &SifrInt::from(args.len()) {
                    let token_opt2_value_c3002fe5b12ff372: Option<String> = {
                        let sifr_generated_checked_read_collection = &args;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(token_opt2_value_c3002fe5b12ff372) = token_opt2_value_c3002fe5b12ff372
                    else {
                        i = &i + &SifrInt::from_i64(1);
                        continue;
                    };
                    let token_many: String = sifr_generated_copy_token(&Some(
                        token_opt2_value_c3002fe5b12ff372.to_owned(),
                    ));
                    if !force_positional
                        && sifr_generated_is_option_like_token(&self.specs, &token_many)
                    {
                        break;
                    }
                    values.push(token_many.to_owned());
                    i = &i + &SifrInt::from_i64(1);
                }
                return (values.to_vec(), i.clone());
            }
            let mut exact: SifrInt = SifrInt::from_i64(1);
            if sifr_generated_is_digit_string(&spec.nargs.clone()) {
                let sifr_generated_try_res: Result<(), ParseError> = (|| {
                    let parsed_count: SifrInt = SifrInt::parse_decimal(
                        &spec.nargs.clone(),
                        ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                    )
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                    if &parsed_count > &SifrInt::from_i64(0) {
                        exact = parsed_count.clone();
                    }
                    Ok(())
                })();
                if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                    let _e = sifr_generated_try_err.clone();
                    exact = SifrInt::from_i64(1);
                }
            }
            let mut count: SifrInt = SifrInt::from_i64(0);
            while &count < &exact && &i < &SifrInt::from(args.len()) {
                let token_opt3_value_c30030e5b12ff525: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let Some(token_opt3_value_c30030e5b12ff525) = token_opt3_value_c30030e5b12ff525
                else {
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                };
                let token_exact: String =
                    sifr_generated_copy_token(&Some(token_opt3_value_c30030e5b12ff525.to_owned()));
                if !force_positional
                    && sifr_generated_is_option_like_token(&self.specs, &token_exact)
                {
                    break;
                }
                values.push(token_exact.to_owned());
                i = &i + &SifrInt::from_i64(1);
                count = &count + &SifrInt::from_i64(1);
            }
            (values.to_vec(), i.clone())
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        pub fn sifr_generated_collect_positional_values(
            &self,
            args: &[String],
            start: &SifrInt,
            spec: &SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec,
            force_positional: bool,
        ) -> (Vec<String>, SifrInt) {
            let mut values: Vec<String> = Vec::new();
            let mut i: SifrInt = start.clone();
            if &i >= &SifrInt::from(args.len()) {
                return (values.to_vec(), i.clone());
            }
            if spec.nargs.clone() == "?" {
                let token_opt: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(token_opt) = token_opt {
                    let token_one_value_6bbe1fb9e813ac55: String =
                        sifr_generated_copy_token(&Some(token_opt.to_owned()));
                    if !force_positional
                        && sifr_generated_is_option_like_token(
                            &self.specs,
                            &token_one_value_6bbe1fb9e813ac55,
                        )
                    {
                        return (values.to_vec(), i.clone());
                    }
                    values.push(token_one_value_6bbe1fb9e813ac55.to_owned());
                }
                return (values.to_vec(), &i + &SifrInt::from_i64(1));
            }
            if spec.nargs.clone() == "*" || spec.nargs.clone() == "+" {
                while &i < &SifrInt::from(args.len()) {
                    let token_opt2_value_c3002fe5b12ff372: Option<String> = {
                        let sifr_generated_checked_read_collection = &args;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(token_opt2_value_c3002fe5b12ff372) = token_opt2_value_c3002fe5b12ff372
                    else {
                        i = &i + &SifrInt::from_i64(1);
                        continue;
                    };
                    let token_many: String = sifr_generated_copy_token(&Some(
                        token_opt2_value_c3002fe5b12ff372.to_owned(),
                    ));
                    if !force_positional
                        && sifr_generated_is_option_like_token(&self.specs, &token_many)
                    {
                        break;
                    }
                    values.push(token_many.to_owned());
                    i = &i + &SifrInt::from_i64(1);
                }
                return (values.to_vec(), i.clone());
            }
            let mut exact: SifrInt = SifrInt::from_i64(1);
            if sifr_generated_is_digit_string(&spec.nargs.clone()) {
                let sifr_generated_try_res: Result<(), ParseError> = (|| {
                    let parsed_count: SifrInt = SifrInt::parse_decimal(
                        &spec.nargs.clone(),
                        ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                    )
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                    if &parsed_count > &SifrInt::from_i64(0) {
                        exact = parsed_count.clone();
                    }
                    Ok(())
                })();
                if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                    let _e = sifr_generated_try_err.clone();
                    exact = SifrInt::from_i64(1);
                }
            }
            let mut count: SifrInt = SifrInt::from_i64(0);
            while &count < &exact && &i < &SifrInt::from(args.len()) {
                let token_opt3_value_c30030e5b12ff525: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(token_opt3) = token_opt3_value_c30030e5b12ff525 {
                    values.push(sifr_generated_copy_token(&Some(token_opt3.to_owned())));
                    count = &count + &SifrInt::from_i64(1);
                }
                i = &i + &SifrInt::from_i64(1);
            }
            (values.to_vec(), i.clone())
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        #[expect(
            clippy::too_many_lines,
            reason = "one generated Rust function preserves one typed Sifr function"
        )]
        pub fn parse_args(
            &self,
            args: &[String],
        ) -> SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
            let mut ns: SifrGeneratedStdlibSifrX2eargparseX2eNamespace =
                SifrGeneratedStdlibSifrX2eargparseX2eNamespace::new();
            for spec in self.specs.iter().cloned() {
                if spec.kind.clone() == "flag" {
                    ns.set_bool(&spec.dest.clone(), false);
                } else if sifr_generated_nargs_is_multi(&spec.nargs.clone())
                    || spec.nargs.clone() == "*"
                    || spec.nargs.clone() == "+"
                {
                    ns.set_list(&spec.dest.clone(), &Vec::new());
                } else {
                    ns.set(&spec.dest.clone(), &spec.default_value.clone());
                }
            }
            if &SifrInt::from(self.subparsers.len()) > &SifrInt::from_i64(0)
                && &SifrInt::from(args.len()) > &SifrInt::from_i64(0)
            {
                let first_token: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(first_token) = first_token {
                    let command_name: String =
                        sifr_generated_copy_token(&Some(first_token.to_owned()));
                    let subparser_specs: Option<
                        Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec>,
                    > = self.sifr_generated_find_subparser(&command_name);
                    if let Some(subparser_specs) = subparser_specs {
                        ns.set(&self.subparsers_dest.clone(), &command_name);
                        let mut subparser: SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser =
                            SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser::new(command_name);
                        subparser.specs = subparser_specs;
                        let child_ns: SifrGeneratedStdlibSifrX2eargparseX2eNamespace = subparser
                            .parse_args(&{
                                let sifr_generated_slice_src = &args;
                                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                                let sifr_generated_slice_start = SifrInt::from_i64(1)
                                    .clamp_slice_bound(sifr_generated_slice_len);
                                let sifr_generated_slice_stop = SifrInt::from(args.len())
                                    .clamp_slice_bound(sifr_generated_slice_len);
                                Vec::from_iter(
                                    sifr_generated_slice_src
                                        .iter()
                                        .skip(sifr_generated_slice_start)
                                        .take(
                                            sifr_generated_slice_stop
                                                .saturating_sub(sifr_generated_slice_start),
                                        )
                                        .cloned(),
                                )
                            });
                        ns.merge_from(&child_ns);
                        return ns;
                    }
                }
            }
            let mut positional_specs: Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec> =
                Vec::new();
            for spec2 in self.specs.iter().cloned() {
                if spec2.kind.clone() == "positional" {
                    positional_specs.push(spec2.clone());
                }
            }
            let mut i: SifrInt = SifrInt::from_i64(0);
            let mut positional_index: SifrInt = SifrInt::from_i64(0);
            let mut force_positional: bool = false;
            while &i < &SifrInt::from(args.len()) {
                let token_opt: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let Some(token_opt_value_6bfb1ab9e84751f4) = token_opt else {
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                };
                let token: String =
                    sifr_generated_copy_token(&Some(token_opt_value_6bfb1ab9e84751f4.to_owned()));
                if token == "--" && !force_positional {
                    force_positional = true;
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                if token.starts_with("-") && !force_positional {
                    let (inline_has_value, inline_name, inline_value) =
                        sifr_generated_split_inline_option(&token);
                    let _chars_inline_name: Vec<char> = inline_name.chars().collect::<Vec<char>>();
                    let _chars_inline_value: Vec<char> =
                        inline_value.chars().collect::<Vec<char>>();
                    let lookup_name: String = if inline_has_value {
                        {
                            let mut sifr_generated_concat: String =
                                String::with_capacity(inline_name.len());
                            sifr_generated_concat.push_str(inline_name.as_str());
                            sifr_generated_concat.push_str("");
                            sifr_generated_concat
                        }
                    } else {
                        let mut sifr_generated_concat: String = String::with_capacity(token.len());
                        sifr_generated_concat.push_str(token.as_str());
                        sifr_generated_concat.push_str("");
                        sifr_generated_concat
                    };
                    let mut handled_option: bool = false;
                    for option_spec in self.specs.iter().cloned() {
                        if option_spec.kind.clone() == "positional" {
                            continue;
                        }
                        if option_spec.name.clone() != lookup_name {
                            continue;
                        }
                        handled_option = true;
                        if option_spec.kind.clone() == "flag" {
                            ns.set_bool(&option_spec.dest.clone(), true);
                            i = &i + &SifrInt::from_i64(1);
                            break;
                        }
                        let values: Vec<String> = if inline_has_value {
                            {
                                let values = vec![inline_value.to_owned()];
                                i = &i + &SifrInt::from_i64(1);
                                values
                            }
                        } else {
                            let (sifr_generated_tuple_unpack_0, sifr_generated_tuple_unpack_1) =
                                self.sifr_generated_collect_option_values(
                                    args,
                                    &(&i + &SifrInt::from_i64(1)),
                                    &option_spec,
                                    force_positional,
                                );
                            let values = sifr_generated_tuple_unpack_0;
                            i = sifr_generated_tuple_unpack_1;
                            values
                        };
                        if sifr_generated_nargs_is_multi(&option_spec.nargs.clone())
                            || option_spec.nargs.clone() == "*"
                            || option_spec.nargs.clone() == "+"
                        {
                            let mut converted_values: Vec<String> = Vec::new();
                            for raw in values.iter().cloned() {
                                let coerced: Option<String> =
                                    self.sifr_generated_coerce_token(&option_spec, &raw);
                                let Some(coerced_value_9a594b45880c48d4) = coerced else {
                                    continue;
                                };
                                converted_values.push(sifr_generated_copy_token(&Some(
                                    coerced_value_9a594b45880c48d4.to_owned(),
                                )));
                            }
                            ns.set_list(&option_spec.dest.clone(), &converted_values);
                        } else if &SifrInt::from(values.len()) > &SifrInt::from_i64(0) {
                            let first_value: Option<String> = {
                                let sifr_generated_checked_read_collection = &values;
                                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                                let sifr_generated_checked_read_normalized =
                                    sifr_generated_checked_read_index.normalize_index_or_len(
                                        sifr_generated_checked_read_collection.len(),
                                    );
                                sifr_generated_checked_read_collection
                                    .get(sifr_generated_checked_read_normalized)
                                    .cloned()
                            };
                            if let Some(first_value) = first_value {
                                let token_value: String =
                                    sifr_generated_copy_token(&Some(first_value.to_owned()));
                                let coerced_first: Option<String> =
                                    self.sifr_generated_coerce_token(&option_spec, &token_value);
                                if let Some(coerced_first) = coerced_first {
                                    let coerced_value: String =
                                        sifr_generated_copy_token(&Some(coerced_first.to_owned()));
                                    ns.set(&option_spec.dest.clone(), &coerced_value);
                                    if option_spec.type_name.clone() == "bool" {
                                        ns.set_bool(
                                            &option_spec.dest.clone(),
                                            coerced_value == "true",
                                        );
                                    }
                                }
                            }
                        }
                        break;
                    }
                    if handled_option {
                        continue;
                    }
                }
                if &positional_index < &SifrInt::from(positional_specs.len()) {
                    let positional_spec_value_f84646974d692a63: Option<
                        SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec,
                    > = {
                        let sifr_generated_checked_read_collection = &positional_specs;
                        let sifr_generated_checked_read_index = positional_index.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(positional_spec) = positional_spec_value_f84646974d692a63 {
                        let (values2_value_a37f29e9b1875a8b, next_i2) = self
                            .sifr_generated_collect_positional_values(
                                args,
                                &i,
                                &positional_spec,
                                force_positional,
                            );
                        if sifr_generated_nargs_is_multi(&positional_spec.nargs.clone())
                            || positional_spec.nargs.clone() == "*"
                            || positional_spec.nargs.clone() == "+"
                        {
                            let mut converted_values2_value_d5873b4bca1f063e: Vec<String> =
                                Vec::new();
                            for raw2 in values2_value_a37f29e9b1875a8b.iter().cloned() {
                                let coerced2_value_5203cd262cdfded2: Option<String> =
                                    self.sifr_generated_coerce_token(&positional_spec, &raw2);
                                let Some(coerced2_value_5203cd262cdfded2) =
                                    coerced2_value_5203cd262cdfded2
                                else {
                                    continue;
                                };
                                converted_values2_value_d5873b4bca1f063e.push(
                                    sifr_generated_copy_token(&Some(
                                        coerced2_value_5203cd262cdfded2.to_owned(),
                                    )),
                                );
                            }
                            ns.set_list(
                                &positional_spec.dest.clone(),
                                &converted_values2_value_d5873b4bca1f063e,
                            );
                        } else if &SifrInt::from(values2_value_a37f29e9b1875a8b.len())
                            > &SifrInt::from_i64(0)
                        {
                            let first_value2_value_418fe1d187bd6a23: Option<String> = {
                                let sifr_generated_checked_read_collection =
                                    &values2_value_a37f29e9b1875a8b;
                                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                                let sifr_generated_checked_read_normalized =
                                    sifr_generated_checked_read_index.normalize_index_or_len(
                                        sifr_generated_checked_read_collection.len(),
                                    );
                                sifr_generated_checked_read_collection
                                    .get(sifr_generated_checked_read_normalized)
                                    .cloned()
                            };
                            if let Some(first_value2) = first_value2_value_418fe1d187bd6a23 {
                                let token_value2_value_5399f4e73b5dc95a: String =
                                    sifr_generated_copy_token(&Some(first_value2.to_owned()));
                                let coerced_first2_value_09853aeb8e001655: Option<String> = self
                                    .sifr_generated_coerce_token(
                                        &positional_spec,
                                        &token_value2_value_5399f4e73b5dc95a,
                                    );
                                if let Some(coerced_first2) = coerced_first2_value_09853aeb8e001655
                                {
                                    let coerced_value2_value_8b96ef5a277fa4a4: String =
                                        sifr_generated_copy_token(&Some(coerced_first2.to_owned()));
                                    ns.set(
                                        &positional_spec.dest.clone(),
                                        &coerced_value2_value_8b96ef5a277fa4a4,
                                    );
                                    if positional_spec.type_name.clone() == "bool" {
                                        ns.set_bool(
                                            &positional_spec.dest.clone(),
                                            coerced_value2_value_8b96ef5a277fa4a4 == "true",
                                        );
                                    }
                                }
                            }
                        }
                        i = next_i2.clone();
                        positional_index = &positional_index + &SifrInt::from_i64(1);
                        continue;
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
            ns
        }
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("CycleError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {}
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        pub nodes: Vec<SifrInt>,
        pub from_nodes: Vec<SifrInt>,
        pub to_nodes: Vec<SifrInt>,
        pub max_node: SifrInt,
        pub prepared: bool,
        pub ready_order: Vec<SifrInt>,
        pub next_index: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        #[must_use]
        pub fn new() -> Self {
            let sifr_generated_field_value_ca4efc7207239f3a_6e6f646573: Vec<SifrInt> = Vec::new();
            let sifr_generated_field_value_e6fd5a19a19860db_66726f6d5f6e6f646573: Vec<SifrInt> =
                Vec::new();
            let sifr_generated_field_value_10a7723d02448bee_746f5f6e6f646573: Vec<SifrInt> =
                Vec::new();
            let sifr_generated_field_value_329212388287f8ee_6d61785f6e6f6465: SifrInt =
                -SifrInt::from_i64(1);
            let sifr_generated_field_value_d2fc88caa16ddddb_5f7072657061726564: bool = false;
            let sifr_generated_field_value_735ddaefc73fa22e_5f72656164795f6f72646572: Vec<SifrInt> =
                Vec::new();
            let sifr_generated_field_value_6b760d8d62496bd0_5f6e6578745f696e646578: SifrInt =
                SifrInt::from_i64(0);
            Self {
                nodes: sifr_generated_field_value_ca4efc7207239f3a_6e6f646573,
                from_nodes: sifr_generated_field_value_e6fd5a19a19860db_66726f6d5f6e6f646573,
                to_nodes: sifr_generated_field_value_10a7723d02448bee_746f5f6e6f646573,
                max_node: sifr_generated_field_value_329212388287f8ee_6d61785f6e6f6465,
                prepared: sifr_generated_field_value_d2fc88caa16ddddb_5f7072657061726564,
                ready_order: sifr_generated_field_value_735ddaefc73fa22e_5f72656164795f6f72646572,
                next_index: sifr_generated_field_value_6b760d8d62496bd0_5f6e6578745f696e646578,
            }
        }
    }
    impl ::std::default::Default for SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        fn default() -> Self {
            Self::new()
        }
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        pub fn sifr_generated_record_node(&mut self, node: &SifrInt) {
            if !sifr_generated_contains_int(&self.nodes, node.clone()) {
                self.nodes.push(node.clone());
            }
            if node > &self.max_node {
                self.max_node = node.clone();
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        pub fn add(&mut self, node: &SifrInt, predecessor: &SifrInt) {
            self.sifr_generated_record_node(node);
            self.sifr_generated_record_node(predecessor);
            self.from_nodes.push(predecessor.clone());
            self.to_nodes.push(node.clone());
            self.prepared = false;
            self.ready_order = Vec::new();
            self.next_index = SifrInt::from_i64(0);
        }
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        pub fn add_many(&mut self, node: &SifrInt, predecessors: &[SifrInt]) {
            self.sifr_generated_record_node(node);
            if &SifrInt::from(predecessors.len()) == &SifrInt::from_i64(0) {
                self.prepared = false;
                self.ready_order = Vec::new();
                self.next_index = SifrInt::from_i64(0);
                return;
            }
            for predecessor in predecessors.iter().cloned() {
                self.add(node, &predecessor);
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        #[must_use]
        pub fn sifr_generated_filter_order(&self, order: &[SifrInt]) -> Vec<SifrInt> {
            let mut filtered: Vec<SifrInt> = Vec::new();
            for candidate in order.iter().cloned() {
                if sifr_generated_contains_int(&self.nodes, candidate.clone()) {
                    filtered.push(candidate.clone());
                }
            }
            filtered
        }
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn static_order(
            &self,
        ) -> Result<Vec<SifrInt>, SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> {
            if &self.max_node.clone() < &SifrInt::from_i64(0) {
                return Ok(Vec::new());
            }
            let sifr_generated_try_res: Result<
                Result<Vec<SifrInt>, SifrGeneratedStdlibSifrX2egraphlibX2eCycleError>,
                SifrGeneratedStdlibSifrX2egraphlibX2eCycleError,
            > = (|| {
                let full_order: Vec<SifrInt> = topological_sort(
                    &self.max_node.clone() + &SifrInt::from_i64(1),
                    &self.from_nodes,
                    &self.to_nodes,
                )?;
                Ok(Ok(self.sifr_generated_filter_order(&full_order)))
            })();
            sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
                let e = sifr_generated_try_err.clone();
                Err(SifrGeneratedStdlibSifrX2egraphlibX2eCycleError::new(
                    e.message.clone(),
                ))
            })
        }
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eipaddressX2eAddressValueError {
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2eipaddressX2eAddressValueError {
        #[must_use]
        pub fn new(message: String) -> Self {
            let sifr_generated_field_value_546401b5d2a8d2a4_6d657373616765: String = {
                let mut sifr_generated_concat: String = String::with_capacity(message.len());
                sifr_generated_concat.push_str(message.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            Self {
                message: sifr_generated_field_value_546401b5d2a8d2a4_6d657373616765,
            }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2eipaddressX2eAddressValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("AddressValueError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eipaddressX2eAddressValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2eipaddressX2eAddressValueError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address {
        pub text: String,
        pub value: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address {
        #[must_use]
        pub fn new(addr: String) -> Self {
            let mut normalized_text: String = {
                let mut sifr_generated_concat: String = String::with_capacity(addr.len());
                sifr_generated_concat.push_str(addr.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let normalized_value: SifrInt = if is_valid_ipv4(&addr) {
                {
                    let parsed: SifrInt = sifr_generated_ip_to_int_raw(&addr);
                    let normalized_value = parsed.clone();
                    normalized_text = int_to_ip(parsed.clone());
                    normalized_value
                }
            } else {
                -&SifrInt::from_i64(1)
            };
            let sifr_generated_field_value_bc70a4514792b60f_5f76616c7565: SifrInt =
                normalized_value.clone();
            let sifr_generated_field_value_c0423f4fcc2bdeed_5f74657874: String = normalized_text;
            Self {
                value: sifr_generated_field_value_bc70a4514792b60f_5f76616c7565,
                text: sifr_generated_field_value_c0423f4fcc2bdeed_5f74657874,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address {
        #[must_use]
        pub fn to_str(&self) -> String {
            {
                let mut sifr_generated_concat: String = String::new();
                sifr_generated_concat.push_str(self.text.clone().as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address {
        #[must_use]
        pub fn is_global(&self) -> bool {
            is_global(&self.text)
        }
    }
    impl SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address {
        #[must_use]
        pub fn is_link_local(&self) -> bool {
            is_link_local(&self.text)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "IPv4Address(_text={}, _value={})", self.text, self.value)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2euuidX2eUUID {
        pub hex: String,
    }
    impl SifrGeneratedStdlibSifrX2euuidX2eUUID {
        #[must_use]
        pub fn new(hex_str: String) -> Self {
            let sifr_generated_field_value_123cb3437a89ad57_5f686578: String = {
                let mut sifr_generated_concat: String = String::with_capacity(hex_str.len());
                sifr_generated_concat.push_str(hex_str.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            Self {
                hex: sifr_generated_field_value_123cb3437a89ad57_5f686578,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2euuidX2eUUID {
        #[must_use]
        pub fn to_str(&self) -> String {
            {
                let mut sifr_generated_concat: String = String::new();
                sifr_generated_concat.push_str(self.hex.clone().as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2euuidX2eUUID {
        #[must_use]
        pub fn version(&self) -> SifrInt {
            let marker: Option<String> = {
                let sifr_generated_string_chars = self.hex.chars().collect::<Vec<char>>();
                let sifr_generated_string_index = SifrInt::from_i64(14);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_string_chars.len());
                sifr_generated_string_chars
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let Some(marker_value_eddcb72b15486e77) = marker else {
                return -&SifrInt::from_i64(1);
            };
            sifr_generated_hex_digit_value(&marker_value_eddcb72b15486e77)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2euuidX2eUUID {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "UUID(_hex={})", self.hex)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ParseError {
        pub message: String,
    }
    impl ::std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ParseError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eargparseX2eNamespace;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2egraphlibX2eCycleError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eipaddressX2eAddressValueError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2euuidX2eUUID;
pub use sifr_generated_project_nominals::ValueError;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let mut parser: SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser =
        SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser::new("e2-demo".to_string());
    (&mut parser).add_argument(
        &"--strict".to_string(),
        &"strict".to_string(),
        &"store_true".to_string(),
        &String::new(),
    );
    (&mut parser).add_argument(
        &"--mode".to_string(),
        &"mode".to_string(),
        &"store".to_string(),
        &"safe".to_string(),
    );
    (&mut parser).add_argument(
        &"entry".to_string(),
        &"entry".to_string(),
        &"store".to_string(),
        &"demo.sifr".to_string(),
    );
    let parsed_value_e06e69d836b17138: SifrGeneratedStdlibSifrX2eargparseX2eNamespace = parser
        .parse_args(&vec![
            "--strict".to_string(),
            "--mode".to_string(),
            "parity".to_string(),
            "main.sifr".to_string(),
        ]);
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(18usize);
        sifr_generated_concat.push_str("argparse.strict = ");
        sifr_generated_concat.push_str(
            parsed_value_e06e69d836b17138
                .get_bool(&"strict".to_string(), false)
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(16usize);
        sifr_generated_concat.push_str("argparse.mode = ");
        sifr_generated_concat.push_str(
            parsed_value_e06e69d836b17138
                .get(&"mode".to_string(), &String::new())
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(17usize);
        sifr_generated_concat.push_str("argparse.entry = ");
        sifr_generated_concat.push_str(
            parsed_value_e06e69d836b17138
                .get(&"entry".to_string(), &String::new())
                .as_str(),
        );
        sifr_generated_concat
    });
    let parsed_inline: SifrGeneratedStdlibSifrX2eargparseX2eNamespace = parser.parse_args(&vec![
        "--mode=inline".to_string(),
        "--".to_string(),
        "--literal.sifr".to_string(),
    ]);
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(18usize);
        sifr_generated_concat.push_str("argparse.inline = ");
        sifr_generated_concat.push_str(
            parsed_inline
                .get(&"mode".to_string(), &String::new())
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(19usize);
        sifr_generated_concat.push_str("argparse.literal = ");
        sifr_generated_concat.push_str(
            parsed_inline
                .get(&"entry".to_string(), &String::new())
                .as_str(),
        );
        sifr_generated_concat
    });
    let parsed_missing: SifrGeneratedStdlibSifrX2eargparseX2eNamespace = parser.parse_args(&vec![
        "--mode".to_string(),
        "--strict".to_string(),
        "fallback.sifr".to_string(),
    ]);
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(24usize);
        sifr_generated_concat.push_str("argparse.missing_mode = ");
        sifr_generated_concat.push_str(
            parsed_missing
                .get(&"mode".to_string(), &String::new())
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(26usize);
        sifr_generated_concat.push_str("argparse.missing_strict = ");
        sifr_generated_concat.push_str(
            parsed_missing
                .get_bool(&"strict".to_string(), false)
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedStdlibSifrX2eipaddressX2eAddressValueError,
    > = (|| {
        let addr: SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address =
            ip_address(&"8.8.8.8".to_string())?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(18usize + 8usize);
            sifr_generated_concat.push_str("ipaddress.value = ");
            sifr_generated_concat.push_str(addr.to_str().as_str());
            sifr_generated_concat.push_str(" global=");
            sifr_generated_concat.push_str(addr.is_global().to_string().as_str());
            sifr_generated_concat
        });
        let link_local: SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address =
            ip_address(&"169.254.10.20".to_string())?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(23usize + 8usize);
            sifr_generated_concat.push_str("ipaddress.link_local = ");
            sifr_generated_concat.push_str(link_local.is_link_local().to_string().as_str());
            sifr_generated_concat.push_str(" global=");
            sifr_generated_concat.push_str(link_local.is_global().to_string().as_str());
            sifr_generated_concat
        });
        let multicast: SifrGeneratedStdlibSifrX2eipaddressX2eIPv4Address =
            ip_address(&"224.0.0.1".to_string())?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(29usize);
            sifr_generated_concat.push_str("ipaddress.multicast_global = ");
            sifr_generated_concat.push_str(multicast.is_global().to_string().as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(18usize);
            sifr_generated_concat.push_str("ipaddress.error = ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let generated: SifrGeneratedStdlibSifrX2euuidX2eUUID = uuid4_obj();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(15usize + 6usize);
        sifr_generated_concat.push_str("uuid.version = ");
        sifr_generated_concat.push_str(generated.version().to_string().as_str());
        sifr_generated_concat.push_str(" text=");
        sifr_generated_concat.push_str(generated.to_str().as_str());
        sifr_generated_concat
    });
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let parsed_curly: SifrGeneratedStdlibSifrX2euuidX2eUUID =
            uuid_from_hex(&"{550E8400-E29B-41D4-A716-446655440000}".to_string())?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(19usize);
            sifr_generated_concat.push_str("uuid.curly.parse = ");
            sifr_generated_concat.push_str(parsed_curly.to_str().as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(13usize);
            sifr_generated_concat.push_str("uuid.error = ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let mut sorter: SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter =
        SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter::new();
    (&mut sorter).add_many(
        &SifrInt::from_i64(50),
        &vec![SifrInt::from_i64(30), SifrInt::from_i64(40)],
    );
    (&mut sorter).add(&SifrInt::from_i64(30), &SifrInt::from_i64(10));
    (&mut sorter).add(&SifrInt::from_i64(40), &SifrInt::from_i64(10));
    (&mut sorter).add_many(&SifrInt::from_i64(10), &Vec::new());
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> =
        (|| {
            let order: Vec<SifrInt> = sorter.static_order()?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(17usize);
                sifr_generated_concat.push_str("graphlib.order = ");
                sifr_generated_concat.push_str(format!("{order:?}").as_str());
                sifr_generated_concat
            });
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(17usize);
            sifr_generated_concat.push_str("graphlib.error = ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
}
