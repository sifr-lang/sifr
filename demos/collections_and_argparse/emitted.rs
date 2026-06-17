use std::collections::HashMap;

// --- stdlib: sifr.argparse ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ArgumentSpec {
    name: String,
    dest: String,
    kind: String,
    default_value: String,
    nargs: String,
    type_name: String,
}
impl ArgumentSpec {
    fn new(
        name: String,
        dest: String,
        kind: String,
        default_value: String,
        nargs: String,
        type_name: String,
    ) -> Self {
        return Self {
            name: format!("{}{}", name, "".to_string()),
            dest: format!("{}{}", dest, "".to_string()),
            kind: format!("{}{}", kind, "".to_string()),
            default_value: format!("{}{}", default_value, "".to_string()),
            nargs: _normalize_nargs(&nargs),
            type_name: format!("{}{}", type_name, "".to_string()),
        };
    }
}
impl std::fmt::Display for ArgumentSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "ArgumentSpec(name={}, dest={}, kind={}, default_value={}, nargs={}, type_name={})",
            self.name, self.dest, self.kind, self.default_value, self.nargs, self
            .type_name
        );
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Namespace {
    _str_values: Vec<(String, String)>,
    _bool_values: Vec<(String, bool)>,
    _list_values: Vec<(String, Vec<String>)>,
}
impl Namespace {
    fn new() -> Self {
        return Self {
            _str_values: vec![],
            _bool_values: vec![],
            _list_values: vec![],
        };
    }
    fn set(&mut self, name: &String, value: &String) {
        let mut updated: Vec<(String, String)> = vec![];
        let mut replaced: bool = false;
        for (key, current) in self._str_values.clone().iter().cloned() {
            if key == *name {
                updated
                    .push((
                        format!("{}{}", name, "".to_string()),
                        format!("{}{}", value, "".to_string()),
                    ));
                replaced = true;
            } else {
                updated.push((key, current));
            }
        }
        if !replaced {
            updated
                .push((
                    format!("{}{}", name, "".to_string()),
                    format!("{}{}", value, "".to_string()),
                ));
        }
        self._str_values = updated;
    }
    fn set_bool(&mut self, name: &String, value: bool) {
        let mut updated: Vec<(String, bool)> = vec![];
        let mut replaced: bool = false;
        for (key, current) in self._bool_values.clone().iter().cloned() {
            if key == *name {
                updated.push((format!("{}{}", name, "".to_string()), value));
                replaced = true;
            } else {
                updated.push((key, current));
            }
        }
        if !replaced {
            updated.push((format!("{}{}", name, "".to_string()), value));
        }
        self._bool_values = updated;
    }
    fn set_list(&mut self, name: &String, values: &Vec<String>) {
        let mut copied: Vec<String> = vec![];
        for value in values.iter().cloned() {
            copied.push(format!("{}{}", value, "".to_string()));
        }
        let mut updated: Vec<(String, Vec<String>)> = vec![];
        for (key, current) in self._list_values.clone().iter().cloned() {
            if key == *name {
                continue;
            }
            updated.push((key, current));
        }
        updated.push((format!("{}{}", name, "".to_string()), copied));
        self._list_values = updated;
    }
    fn get(&self, name: &String, default: &String) -> String {
        for (key, value) in self._str_values.clone().iter().cloned() {
            if key == *name {
                return format!("{}{}", value, "".to_string());
            }
        }
        return format!("{}{}", default, "".to_string());
    }
    fn get_bool(&self, name: &String, default: bool) -> bool {
        for (key, value) in self._bool_values.clone().iter().cloned() {
            if key == *name {
                return value;
            }
        }
        for (key2, value2) in self._str_values.clone().iter().cloned() {
            if key2 != *name {
                continue;
            }
            let normalized: String = value2.to_lowercase();
            if (((normalized == "1".to_string()) || (normalized == "true".to_string()))
                || (normalized == "yes".to_string())) || (normalized == "on".to_string())
            {
                return true;
            }
            if (((normalized == "0".to_string()) || (normalized == "false".to_string()))
                || (normalized == "no".to_string())) || (normalized == "off".to_string())
            {
                return false;
            }
        }
        return default;
    }
    fn get_list(&self, name: &String) -> Vec<String> {
        for (key, values) in self._list_values.clone().iter().cloned() {
            if key != *name {
                continue;
            }
            let mut copied: Vec<String> = vec![];
            for value in values.iter().cloned() {
                copied.push(format!("{}{}", value, "".to_string()));
            }
            return copied;
        }
        return vec![];
    }
    fn merge_from(&mut self, other: &Namespace) {
        for (key, value) in other._str_values.iter().cloned() {
            self.set(&key, &value);
        }
        for (key2, value2) in other._bool_values.iter().cloned() {
            self.set_bool(&key2, value2);
        }
        for (key3, values3) in other._list_values.iter().cloned() {
            self.set_list(&key3, &values3);
        }
    }
    fn copy(&self) -> Namespace {
        let mut copied: Namespace = Namespace::new();
        copied.merge_from(&self);
        return copied;
    }
}
#[derive(Debug, Clone, PartialEq)]
struct ArgumentParser {
    _prog: String,
    _specs: Vec<ArgumentSpec>,
    _subparsers_dest: String,
    _subparsers: Vec<(String, Vec<ArgumentSpec>)>,
}
impl ArgumentParser {
    fn new(prog: String) -> Self {
        return Self {
            _prog: format!("{}{}", prog, "".to_string()),
            _specs: vec![],
            _subparsers_dest: "command".to_string(),
            _subparsers: vec![],
        };
    }
    fn prog(&self) -> String {
        return format!("{}{}", self._prog.clone(), "".to_string());
    }
    fn add_subparsers(&mut self, dest: &String) {
        if dest.clone() != "".to_string() {
            self._subparsers_dest = format!("{}{}", dest, "".to_string());
        }
    }
    fn add_parser(&mut self, name: &String, parser: &ArgumentParser) {
        let mut specs_copy: Vec<ArgumentSpec> = vec![];
        for spec in parser._specs.iter().cloned() {
            specs_copy.push(spec);
        }
        self._subparsers.push((format!("{}{}", name, "".to_string()), specs_copy));
    }
    fn _append_spec(
        &mut self,
        name: &String,
        dest: &String,
        action: &String,
        default: &String,
        nargs: &String,
        type_name: &String,
    ) {
        let mut resolved_dest: String = format!("{}{}", dest, "".to_string());
        if resolved_dest == "".to_string() {
            resolved_dest = _derive_dest(name);
        }
        let mut kind: String = "positional".to_string();
        if name.starts_with(&"-".to_string()) {
            if action.clone() == "store_true".to_string() {
                kind = "flag".to_string();
            } else {
                kind = "option".to_string();
            }
        }
        let spec: ArgumentSpec = ArgumentSpec::new(
            (name).clone(),
            resolved_dest,
            kind,
            (default).clone(),
            (nargs).clone(),
            (type_name).clone(),
        );
        self._specs.push(spec);
    }
    fn add_argument(
        &mut self,
        name: &String,
        dest: &String,
        action: &String,
        default: &String,
    ) {
        self.add_argument_typed(
            name,
            dest,
            action,
            default,
            &"1".to_string(),
            &"str".to_string(),
        );
    }
    fn add_argument_typed(
        &mut self,
        name: &String,
        dest: &String,
        action: &String,
        default: &String,
        nargs: &String,
        type_name: &String,
    ) {
        let mut normalized_type: String = format!("{}{}", type_name, "".to_string());
        if (((normalized_type != "int".to_string())
            && (normalized_type != "float".to_string()))
            && (normalized_type != "bool".to_string()))
            && (normalized_type != "str".to_string())
        {
            normalized_type = "str".to_string();
        }
        self._append_spec(name, dest, action, default, nargs, &normalized_type);
    }
    fn _find_subparser(&self, name: &String) -> Option<Vec<ArgumentSpec>> {
        for (parser_name, parser_specs) in self._subparsers.clone().iter().cloned() {
            if parser_name == *name {
                return Some(parser_specs);
            }
        }
        return None;
    }
    fn _coerce_token(&self, spec: &ArgumentSpec, token: &String) -> Option<String> {
        if spec.type_name == "int".to_string() {
            let __sifr_try_res: Result<Option<String>, ParseError> = (|| {
                let parsed_int: i64 = (token)
                    .parse::<i64>()
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                return Ok(Some(format!("{}", parsed_int)));
                unreachable!("sifr try/except return capture fell through");
            })();
            match __sifr_try_res {
                Ok(__sifr_ret_val) => {
                    return __sifr_ret_val;
                }
                Err(__sifr_try_err) => {
                    let _e = __sifr_try_err.clone();
                    return None;
                }
            }
        }
        if spec.type_name == "float".to_string() {
            let __sifr_try_res: Result<Option<String>, ParseError> = (|| {
                let parsed_float: f64 = (token)
                    .parse::<f64>()
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                return Ok(Some(format!("{}", parsed_float)));
                unreachable!("sifr try/except return capture fell through");
            })();
            match __sifr_try_res {
                Ok(__sifr_ret_val) => {
                    return __sifr_ret_val;
                }
                Err(__sifr_try_err) => {
                    let _e = __sifr_try_err.clone();
                    return None;
                }
            }
        }
        if spec.type_name == "bool".to_string() {
            return _coerce_bool(token);
        }
        return Some(format!("{}{}", token, "".to_string()));
    }
    fn _collect_option_values(
        &self,
        args: &Vec<String>,
        start: i64,
        spec: &ArgumentSpec,
        force_positional: bool,
    ) -> (Vec<String>, i64) {
        let mut values: Vec<String> = vec![];
        let mut i: i64 = start;
        if spec.nargs == "?".to_string() {
            if i >= (args.len() as i64) {
                return (values, i);
            }
            let token_opt: Option<String> = Some(args[i as usize].clone());
            let Some(token_opt) = token_opt else {
                return (values, i + (1 as i64));
            };
            let token_one: String = _copy_token(&Some(token_opt));
            if ((!(force_positional))
                && (_is_option_like_token(&self._specs.clone(), &token_one)))
            {
                return (values, i);
            }
            values.push(token_one);
            return (values, i + (1 as i64));
        }
        if ((spec.nargs == "*".to_string()) || (spec.nargs == "+".to_string())) {
            while i < (args.len() as i64) {
                let token_opt2: Option<String> = Some(args[i as usize].clone());
                if token_opt2.is_none() {
                    i = i + (1 as i64);
                    continue;
                }
                let token_many: String = _copy_token(&token_opt2);
                if ((!(force_positional))
                    && (_is_option_like_token(&self._specs.clone(), &token_many)))
                {
                    break;
                }
                values.push(token_many);
                i = i + (1 as i64);
            }
            return (values, i);
        }
        let mut exact: i64 = 1 as i64;
        if _is_digit_string(&spec.nargs) {
            let __sifr_try_res: Result<(), ParseError> = (|| {
                let parsed_count: i64 = (spec.nargs)
                    .parse::<i64>()
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                if parsed_count > (0 as i64) {
                    exact = parsed_count;
                }
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let _e = __sifr_try_err.clone();
                exact = 1 as i64;
            }
        }
        let mut count: i64 = 0 as i64;
        while ((count < exact) && (i < (args.len() as i64))) {
            let token_opt3: Option<String> = Some(args[i as usize].clone());
            if token_opt3.is_none() {
                i = i + (1 as i64);
                continue;
            }
            let token_exact: String = _copy_token(&token_opt3);
            if ((!(force_positional))
                && (_is_option_like_token(&self._specs.clone(), &token_exact)))
            {
                break;
            }
            values.push(token_exact);
            i = i + (1 as i64);
            count = count + (1 as i64);
        }
        return (values, i);
    }
    fn _collect_positional_values(
        &self,
        args: &Vec<String>,
        start: i64,
        spec: &ArgumentSpec,
        force_positional: bool,
    ) -> (Vec<String>, i64) {
        let mut values: Vec<String> = vec![];
        let mut i: i64 = start;
        if i >= (args.len() as i64) {
            return (values, i);
        }
        if spec.nargs == "?".to_string() {
            let token_opt: Option<String> = Some(args[i as usize].clone());
            if let Some(token_opt) = token_opt {
                let token_one: String = _copy_token(&Some(token_opt));
                if ((!(force_positional))
                    && (_is_option_like_token(&self._specs.clone(), &token_one)))
                {
                    return (values, i);
                }
                values.push(token_one);
            }
            return (values, i + (1 as i64));
        }
        if ((spec.nargs == "*".to_string()) || (spec.nargs == "+".to_string())) {
            while i < (args.len() as i64) {
                let token_opt2: Option<String> = Some(args[i as usize].clone());
                if token_opt2.is_none() {
                    i = i + (1 as i64);
                    continue;
                }
                let token_many: String = _copy_token(&token_opt2);
                if ((!(force_positional))
                    && (_is_option_like_token(&self._specs.clone(), &token_many)))
                {
                    break;
                }
                values.push(token_many);
                i = i + (1 as i64);
            }
            return (values, i);
        }
        let mut exact: i64 = 1 as i64;
        if _is_digit_string(&spec.nargs) {
            let __sifr_try_res: Result<(), ParseError> = (|| {
                let parsed_count: i64 = (spec.nargs)
                    .parse::<i64>()
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                if parsed_count > (0 as i64) {
                    exact = parsed_count;
                }
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let _e = __sifr_try_err.clone();
                exact = 1 as i64;
            }
        }
        let mut count: i64 = 0 as i64;
        while ((count < exact) && (i < (args.len() as i64))) {
            let token_opt3: Option<String> = Some(args[i as usize].clone());
            if let Some(token_opt3) = token_opt3 {
                values.push(_copy_token(&Some(token_opt3)));
                count = count + (1 as i64);
            }
            i = i + (1 as i64);
        }
        return (values, i);
    }
    fn parse_args(&mut self, args: &Vec<String>) -> Namespace {
        let mut ns: Namespace = Namespace::new();
        for spec in self._specs.clone().iter().cloned() {
            if spec.kind == "flag".to_string() {
                ns.set_bool(&spec.dest, false);
            } else {
                if (((_nargs_is_multi(&spec.nargs)) || (spec.nargs == "*".to_string()))
                    || (spec.nargs == "+".to_string()))
                {
                    ns.set_list(&spec.dest, &vec![]);
                } else {
                    ns.set(&spec.dest, &spec.default_value);
                }
            }
        }
        if (((self._subparsers.clone().len() as i64) > (0 as i64))
            && ((args.len() as i64) > (0 as i64)))
        {
            let first_token: Option<String> = Some(args[(0 as i64) as usize].clone());
            if let Some(first_token) = first_token {
                let command_name: String = _copy_token(&Some(first_token));
                let subparser_specs: Option<Vec<ArgumentSpec>> = self
                    ._find_subparser(&command_name);
                if let Some(subparser_specs) = subparser_specs {
                    ns.set(&self._subparsers_dest.clone(), &command_name);
                    let mut subparser: ArgumentParser = ArgumentParser::new(
                        command_name,
                    );
                    subparser._specs = subparser_specs;
                    let child_ns: Namespace = subparser
                        .parse_args(
                            &Vec::from_iter(
                                (args)
                                    .iter()
                                    .skip((1 as i64).max(0) as usize)
                                    .take(
                                        ((args.len() as i64).max(0) - (1 as i64).max(0)).max(0)
                                            as usize,
                                    )
                                    .cloned(),
                            ),
                        );
                    ns.merge_from(&child_ns);
                    return ns;
                }
            }
        }
        let mut positional_specs: Vec<ArgumentSpec> = vec![];
        for spec2 in self._specs.clone().iter().cloned() {
            if spec2.kind == "positional".to_string() {
                positional_specs.push(spec2);
            }
        }
        let mut i: i64 = 0 as i64;
        let mut positional_index: i64 = 0 as i64;
        let mut force_positional: bool = false;
        while i < (args.len() as i64) {
            let token_opt: Option<String> = Some(args[i as usize].clone());
            if token_opt.is_none() {
                i = i + (1 as i64);
                continue;
            }
            let token: String = _copy_token(&token_opt);
            if (token == "--".to_string()) && !force_positional {
                force_positional = true;
                i = i + (1 as i64);
                continue;
            }
            if ((token.starts_with(&"-".to_string())) && (!(force_positional))) {
                let (inline_has_value, inline_name, inline_value) = _split_inline_option(
                    &token,
                );
                let mut lookup_name: String = format!("{}{}", token, "".to_string());
                if inline_has_value {
                    lookup_name = format!("{}{}", inline_name, "".to_string());
                }
                let mut handled_option: bool = false;
                for option_spec in self._specs.clone().iter().cloned() {
                    if option_spec.kind == "positional".to_string() {
                        continue;
                    }
                    if option_spec.name != lookup_name {
                        continue;
                    }
                    handled_option = true;
                    if option_spec.kind == "flag".to_string() {
                        ns.set_bool(&option_spec.dest, true);
                        i = i + (1 as i64);
                        break;
                    }
                    let mut values: Vec<String> = vec![];
                    if inline_has_value {
                        values = vec![inline_value];
                        i = i + (1 as i64);
                    } else {
                        let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1) = self
                            ._collect_option_values(
                                args,
                                i + (1 as i64),
                                &option_spec,
                                force_positional,
                            );
                        values = __sifr_tuple_unpack_0;
                        i = __sifr_tuple_unpack_1;
                    }
                    if (((_nargs_is_multi(&option_spec.nargs))
                        || (option_spec.nargs == "*".to_string()))
                        || (option_spec.nargs == "+".to_string()))
                    {
                        let mut converted_values: Vec<String> = vec![];
                        for raw in values.iter().cloned() {
                            let coerced: Option<String> = self
                                ._coerce_token(&option_spec, &raw);
                            if coerced.is_none() {
                                continue;
                            }
                            converted_values.push(_copy_token(&coerced));
                        }
                        ns.set_list(&option_spec.dest, &converted_values);
                    } else {
                        if (values.len() as i64) > (0 as i64) {
                            let first_value: Option<String> = Some(
                                values[(0 as i64) as usize].clone(),
                            );
                            if let Some(first_value) = first_value {
                                let token_value: String = _copy_token(&Some(first_value));
                                let coerced_first: Option<String> = self
                                    ._coerce_token(&option_spec, &token_value);
                                if let Some(coerced_first) = coerced_first {
                                    let coerced_value: String = _copy_token(
                                        &Some(coerced_first),
                                    );
                                    ns.set(&option_spec.dest, &coerced_value);
                                    if option_spec.type_name == "bool".to_string() {
                                        ns.set_bool(
                                            &option_spec.dest,
                                            coerced_value == "true".to_string(),
                                        );
                                    }
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
            if positional_index < (positional_specs.len() as i64) {
                let positional_spec: Option<ArgumentSpec> = Some(
                    positional_specs[positional_index as usize].clone(),
                );
                if let Some(positional_spec) = positional_spec {
                    let (values2, next_i2) = self
                        ._collect_positional_values(
                            args,
                            i,
                            &positional_spec,
                            force_positional,
                        );
                    if (((_nargs_is_multi(&positional_spec.nargs))
                        || (positional_spec.nargs == "*".to_string()))
                        || (positional_spec.nargs == "+".to_string()))
                    {
                        let mut converted_values2: Vec<String> = vec![];
                        for raw2 in values2.iter().cloned() {
                            let coerced2: Option<String> = self
                                ._coerce_token(&positional_spec, &raw2);
                            if coerced2.is_none() {
                                continue;
                            }
                            converted_values2.push(_copy_token(&coerced2));
                        }
                        ns.set_list(&positional_spec.dest, &converted_values2);
                    } else {
                        if (values2.len() as i64) > (0 as i64) {
                            let first_value2: Option<String> = Some(
                                values2[(0 as i64) as usize].clone(),
                            );
                            if let Some(first_value2) = first_value2 {
                                let token_value2: String = _copy_token(&Some(first_value2));
                                let coerced_first2: Option<String> = self
                                    ._coerce_token(&positional_spec, &token_value2);
                                if let Some(coerced_first2) = coerced_first2 {
                                    let coerced_value2: String = _copy_token(
                                        &Some(coerced_first2),
                                    );
                                    ns.set(&positional_spec.dest, &coerced_value2);
                                    if positional_spec.type_name == "bool".to_string() {
                                        ns.set_bool(
                                            &positional_spec.dest,
                                            coerced_value2 == "true".to_string(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    i = next_i2;
                    positional_index = positional_index + (1 as i64);
                    continue;
                }
            }
            i = i + (1 as i64);
        }
        return ns;
    }
}
fn _split_inline_option(token: &String) -> (bool, String, String) {
    let mut key: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < (token.chars().count() as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = token.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if ((ch != None) && (ch == Some("=".to_string()))) {
            let mut value: String = "".to_string();
            let mut j: i64 = i + (1 as i64);
            while j < (token.chars().count() as i64) {
                let part: Option<String> = Some({
                    let Some(__indexed_char) = token.chars().nth(j as usize) else {
                        unreachable!(
                            "compiler-verified string index should be in range"
                        );
                    };
                    __indexed_char.to_string()
                });
                if let Some(part) = part {
                    value = format!("{}{}", value, part);
                }
                j = j + (1 as i64);
            }
            return (true, key, value);
        }
        if let Some(ch) = ch {
            key = format!("{}{}", key, ch);
        }
        i = i + (1 as i64);
    }
    return (false, format!("{}{}", token, "".to_string()), "".to_string());
}
fn _is_digit_string(value: &String) -> bool {
    if value.clone() == "".to_string() {
        return false;
    }
    for ch in value.chars().map(|c| c.to_string()) {
        if (ch < "0".to_string()) || (ch > "9".to_string()) {
            return false;
        }
    }
    return true;
}
fn _normalize_nargs(nargs: &String) -> String {
    if nargs.clone() == "".to_string() {
        return "1".to_string();
    }
    if (((nargs.clone() == "?".to_string()) || (nargs.clone() == "*".to_string()))
        || (nargs.clone() == "+".to_string()))
    {
        return format!("{}{}", nargs, "".to_string());
    }
    if _is_digit_string(nargs) {
        let __sifr_try_res: Result<Option<String>, ParseError> = (|| {
            let parsed: i64 = (nargs)
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            if parsed > (0 as i64) {
                return Ok(Some(format!("{}", parsed)));
            }
            return Ok(None);
        })();
        match __sifr_try_res {
            Ok(Some(__sifr_ret_val)) => {
                return __sifr_ret_val;
            }
            Ok(None) => {}
            Err(__sifr_try_err) => {
                let _e = __sifr_try_err.clone();
                return "1".to_string();
            }
        }
    }
    return "1".to_string();
}
fn _nargs_is_multi(nargs: &String) -> bool {
    let normalized: String = _normalize_nargs(nargs);
    if (normalized == "*".to_string()) || (normalized == "+".to_string()) {
        return true;
    }
    if _is_digit_string(&normalized) {
        let __sifr_try_res: Result<bool, ParseError> = (|| {
            let parsed: i64 = (normalized)
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            return Ok(parsed > (1 as i64));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let _e = __sifr_try_err.clone();
                return false;
            }
        }
    }
    return false;
}
fn _coerce_bool(raw: &String) -> Option<String> {
    let normalized: String = raw.to_lowercase();
    if (((normalized == "1".to_string()) || (normalized == "true".to_string()))
        || (normalized == "yes".to_string())) || (normalized == "on".to_string())
    {
        return Some("true".to_string());
    }
    if (((normalized == "0".to_string()) || (normalized == "false".to_string()))
        || (normalized == "no".to_string())) || (normalized == "off".to_string())
    {
        return Some("false".to_string());
    }
    return None;
}
fn _copy_token(value: &Option<String>) -> String {
    let Some(value) = value else {
        return "".to_string();
    };
    return format!("{}{}", value, "".to_string());
}
fn _derive_dest(name: &String) -> String {
    if name.starts_with(&"--".to_string()) {
        return name
            .chars()
            .skip((2 as i64) as usize)
            .take(((name.chars().count() as i64) as usize) - ((2 as i64) as usize))
            .collect::<String>()
            .replace(&"-".to_string(), &"_".to_string());
    }
    if name.starts_with(&"-".to_string()) {
        return name
            .chars()
            .skip((1 as i64) as usize)
            .take(((name.chars().count() as i64) as usize) - ((1 as i64) as usize))
            .collect::<String>()
            .replace(&"-".to_string(), &"_".to_string());
    }
    return format!("{}{}", name, "".to_string());
}
fn _is_option_like_token(specs: &Vec<ArgumentSpec>, token: &String) -> bool {
    if token.clone() == "--".to_string() {
        return true;
    }
    if token.starts_with(&"--".to_string()) {
        return true;
    }
    let (inline_has_value, inline_name, inline_value) = _split_inline_option(token);
    let _: String = inline_value;
    let mut lookup_name: String = format!("{}{}", token, "".to_string());
    if inline_has_value {
        lookup_name = format!("{}{}", inline_name, "".to_string());
    }
    for spec in specs.iter().cloned() {
        if spec.kind == "positional".to_string() {
            continue;
        }
        if spec.name == lookup_name {
            return true;
        }
    }
    return false;
}

// --- stdlib: sifr.collections ---
#[derive(Debug, Clone, PartialEq)]
struct Counter<T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq> {
    counts: HashMap<T, i64>,
}
impl<T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq> Counter<T> {
    fn new(source: Option<HashMap<T, i64>>, iterable: Option<Vec<T>>) -> Self {
        let mut counts: HashMap<T, i64> = HashMap::from([]);
        if let Some(source) = source {
            for key in source.keys().cloned().collect::<Vec<_>>() {
                let value: Option<i64> = source.get(&key).copied();
                if let Some(value) = value {
                    counts.insert(key, value);
                }
            }
        }
        if let Some(iterable) = iterable {
            for item in iterable.iter().cloned() {
                let value2: Option<i64> = counts.get(&item).copied();
                if let Some(value2) = value2 {
                    counts.insert(item, value2 + (1 as i64));
                } else {
                    counts.insert(item, 1 as i64);
                }
            }
        }
        return Self { counts: counts };
    }
    fn get(&self, key: &T, default: i64) -> i64 {
        let val: Option<i64> = self.counts.get(&key).copied();
        if let Some(val) = val {
            return val;
        }
        return default;
    }
    fn increment(&mut self, key: &T) {
        let val: Option<i64> = self.counts.get(&key).copied();
        if let Some(val) = val {
            self.counts.insert(key.clone(), val + (1 as i64));
        } else {
            self.counts.insert(key.clone(), 1 as i64);
        }
    }
    fn total(&self) -> i64 {
        let mut total: i64 = 0 as i64;
        for count in self.counts.clone().values().cloned().collect::<Vec<_>>() {
            total = total + count;
        }
        return total;
    }
    fn most_common(&self, n: Option<i64>) -> Vec<(T, i64)> {
        let mut result: Vec<(T, i64)> = vec![];
        for key in self.counts.clone().keys().cloned().collect::<Vec<_>>() {
            let count: Option<i64> = self.counts.get(&key).copied();
            if let Some(count) = count {
                let entry: (T, i64) = (key, count);
                result.push(entry);
            }
        }
        let sz: i64 = result.len() as i64;
        let mut i: i64 = 0 as i64;
        while i < sz {
            let mut j: i64 = i + (1 as i64);
            while j < sz {
                let left: Option<(T, i64)> = {
                    let __sifr_index_list = &result;
                    let __sifr_index_i = i;
                    let __sifr_index_norm = if __sifr_index_i < 0 {
                        ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                    } else {
                        __sifr_index_i as usize
                    };
                    __sifr_index_list.get(__sifr_index_norm).cloned()
                };
                let right: Option<(T, i64)> = {
                    let __sifr_index_list = &result;
                    let __sifr_index_i = j;
                    let __sifr_index_norm = if __sifr_index_i < 0 {
                        ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                    } else {
                        __sifr_index_i as usize
                    };
                    __sifr_index_list.get(__sifr_index_norm).cloned()
                };
                if let Some(left) = left {
                    if let Some(right) = right {
                        if (right).1 > (left).1 {
                            {
                                let __idx_raw = i;
                                let __idx_norm = if __idx_raw < 0 {
                                    (result.len() as i64) + __idx_raw
                                } else {
                                    __idx_raw
                                };
                                if __idx_norm >= 0 {
                                    if let Some(__elem) = result.get_mut(__idx_norm as usize) {
                                        *__elem = right;
                                    }
                                }
                            }
                            {
                                let __idx_raw = j;
                                let __idx_norm = if __idx_raw < 0 {
                                    (result.len() as i64) + __idx_raw
                                } else {
                                    __idx_raw
                                };
                                if __idx_norm >= 0 {
                                    if let Some(__elem) = result.get_mut(__idx_norm as usize) {
                                        *__elem = left;
                                    }
                                }
                            }
                        }
                    }
                }
                j = j + (1 as i64);
            }
            i = i + (1 as i64);
        }
        let Some(n) = n else {
            return result;
        };
        if n <= (0 as i64) {
            return vec![];
        }
        let mut top: Vec<(T, i64)> = vec![];
        let mut index: i64 = 0 as i64;
        while index < n {
            if index >= (result.len() as i64) {
                return top;
            }
            let value: Option<(T, i64)> = Some(result[index as usize].clone());
            if let Some(value) = value {
                top.push(value);
            }
            index = index + (1 as i64);
        }
        return top;
    }
    fn keys(&self) -> Vec<T> {
        return self.counts.clone().keys().cloned().collect::<Vec<_>>();
    }
    fn items(&self) -> Vec<(T, i64)> {
        let mut result: Vec<(T, i64)> = vec![];
        for key in self.counts.clone().keys().cloned().collect::<Vec<_>>() {
            let value: Option<i64> = self.counts.get(&key).copied();
            if let Some(value) = value {
                let entry: (T, i64) = (key, value);
                result.push(entry);
            }
        }
        return result;
    }
    fn values(&self) -> Vec<i64> {
        return self.counts.clone().values().cloned().collect::<Vec<_>>();
    }
    fn copy(&self) -> Counter<T> {
        return Counter::new(Some(self.counts.clone()), None);
    }
    fn clear(&mut self) {
        self.counts = HashMap::from([]);
    }
    fn update(&mut self, other: &Counter<T>) {
        for key in other.counts.keys().cloned().collect::<Vec<_>>() {
            let other_val: Option<i64> = other.counts.get(&key).copied();
            if let Some(other_val) = other_val {
                let existing: Option<i64> = self.counts.get(&key).copied();
                if let Some(existing) = existing {
                    self.counts.insert(key, existing + other_val);
                } else {
                    self.counts.insert(key, other_val);
                }
            }
        }
    }
    fn subtract(&mut self, other: &Counter<T>) {
        for key in other.counts.keys().cloned().collect::<Vec<_>>() {
            let other_val: Option<i64> = other.counts.get(&key).copied();
            if let Some(other_val) = other_val {
                let existing: Option<i64> = self.counts.get(&key).copied();
                if let Some(existing) = existing {
                    self.counts.insert(key, existing - other_val);
                } else {
                    self.counts.insert(key, (0 as i64) - other_val);
                }
            }
        }
    }
    fn elements(&self) -> Vec<T> {
        let mut result: Vec<T> = vec![];
        let all_keys: Vec<T> = self.counts.clone().keys().cloned().collect::<Vec<_>>();
        let mut ki: i64 = 0 as i64;
        while ki < (all_keys.len() as i64) {
            let key_opt: Option<T> = Some(all_keys[ki as usize].clone());
            if let Some(key_opt) = key_opt {
                let cnt: Option<i64> = self.counts.get(&key_opt).copied();
                if let Some(cnt) = cnt {
                    let mut i: i64 = 0 as i64;
                    while i < cnt {
                        let key_copy: Option<T> = Some(all_keys[ki as usize].clone());
                        if let Some(key_copy) = key_copy {
                            result.push(key_copy.clone());
                        }
                        i = i + (1 as i64);
                    }
                }
            }
            ki = ki + (1 as i64);
        }
        return result;
    }
}
impl<
    T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq,
> std::ops::Add<&Counter<T>> for &Counter<T> {
    type Output = Counter<T>;
    fn add(self, other: &Counter<T>) -> Self::Output {
        let mut new_counts: HashMap<T, i64> = HashMap::from([]);
        for key in Box::new(
            (self.counts.clone().keys().cloned().collect::<Vec<_>>()).into_iter(),
        ) {
            let a_val: Option<i64> = self.counts.get(&key).copied();
            if let Some(a_val) = a_val {
                let b_val: Option<i64> = other.counts.get(&key).copied();
                let mut b_count: i64 = 0 as i64;
                if let Some(b_val) = b_val {
                    b_count = b_val;
                }
                let total: i64 = a_val + b_count;
                if total > (0 as i64) {
                    new_counts.insert(key, total);
                }
            }
        }
        for key2 in Box::new(
            (other.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
        ) {
            let already: Option<i64> = new_counts.get(&key2).copied();
            if already == None {
                let b_val2: Option<i64> = other.counts.get(&key2).copied();
                if let Some(b_val2) = b_val2 {
                    if b_val2 > (0 as i64) {
                        new_counts.insert(key2, b_val2);
                    }
                }
            }
        }
        return Counter::new(Some(new_counts), None);
    }
}
impl<
    T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq,
> std::ops::Sub<&Counter<T>> for &Counter<T> {
    type Output = Counter<T>;
    fn sub(self, other: &Counter<T>) -> Self::Output {
        let mut new_counts: HashMap<T, i64> = HashMap::from([]);
        for key in Box::new(
            (self.counts.clone().keys().cloned().collect::<Vec<_>>()).into_iter(),
        ) {
            let a_val: Option<i64> = self.counts.get(&key).copied();
            if let Some(a_val) = a_val {
                let b_val: Option<i64> = other.counts.get(&key).copied();
                let mut b_count: i64 = 0 as i64;
                if let Some(b_val) = b_val {
                    b_count = b_val;
                }
                let diff: i64 = a_val - b_count;
                if diff > (0 as i64) {
                    new_counts.insert(key, diff);
                }
            }
        }
        return Counter::new(Some(new_counts), None);
    }
}
#[derive(Debug, Clone, PartialEq)]
struct defaultdict<K: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq> {
    default_factory: i64,
    _values: HashMap<K, i64>,
}
impl<K: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq> defaultdict<K> {
    fn new(default_factory: i64, initial: Option<HashMap<K, i64>>) -> Self {
        let mut values: HashMap<K, i64> = HashMap::from([]);
        if let Some(initial) = initial {
            for (key, value) in initial
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                values.insert(key, value);
            }
        }
        return Self {
            default_factory: default_factory,
            _values: values,
        };
    }
    fn ensure(&mut self, key: &K) -> i64 {
        let current: Option<i64> = self._values.get(&key).copied();
        if let Some(current) = current {
            return current;
        }
        self._values.insert(key.clone(), self.default_factory);
        let created: Option<i64> = self._values.get(&key).copied();
        if let Some(created) = created {
            return created;
        }
        return self.default_factory;
    }
    fn set(&mut self, key: &K, value: i64) {
        self._values.insert(key.clone(), value);
    }
    fn has(&self, key: &K) -> bool {
        return (self._values.clone()).contains_key(key);
    }
    fn pop(&mut self, key: &K) -> Option<i64> {
        if (self._values.clone()).contains_key(key) {
            return self._values.remove(&key);
        }
        return None;
    }
    fn clear(&mut self) {
        self._values = HashMap::from([]);
    }
    fn keys(&self) -> Vec<K> {
        return self._values.clone().keys().cloned().collect::<Vec<_>>();
    }
    fn values(&self) -> Vec<i64> {
        return self._values.clone().values().cloned().collect::<Vec<_>>();
    }
    fn items(&self) -> Vec<(K, i64)> {
        let mut values: Vec<(K, i64)> = vec![];
        for (key, value) in self
            ._values
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            values.push((key, value));
        }
        return values;
    }
    fn len(&self) -> i64 {
        return self._values.clone().len() as i64;
    }
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ParseError {
}

fn main() {
    let mut counter = Counter::new(None, Some(vec!["parse".to_string(), "parse".to_string(), "emit".to_string()]));
    assert!(counter.get(&"parse".to_string(), 0 as i64) == (2 as i64));
    let mut attempts = defaultdict::new(0 as i64, None);
    let current_attempts: i64 = attempts.ensure(&"collections_and_argparse".to_string());
    attempts.set(&"collections_and_argparse".to_string(), current_attempts + (1 as i64));
    assert!(attempts.ensure(&"collections_and_argparse".to_string()) == (1 as i64));
    let mut parser: ArgumentParser = ArgumentParser::new("sifr".to_string());
    parser.add_subparsers(&"cmd".to_string());
    let mut run_parser: ArgumentParser = ArgumentParser::new("run".to_string());
    run_parser.add_argument_typed(&"--strict".to_string(), &"strict".to_string(), &"store_true".to_string(), &"".to_string(), &"1".to_string(), &"str".to_string());
    run_parser.add_argument_typed(&"--level".to_string(), &"level".to_string(), &"store".to_string(), &"0".to_string(), &"1".to_string(), &"int".to_string());
    run_parser.add_argument_typed(&"--custom-level".to_string(), &"custom_level".to_string(), &"store".to_string(), &"0".to_string(), &"1".to_string(), &"int".to_string());
    run_parser.add_argument_typed(&"targets".to_string(), &"targets".to_string(), &"store".to_string(), &"".to_string(), &"+".to_string(), &"str".to_string());
    parser.add_parser(&"run".to_string(), &run_parser);
    let mut parsed: Namespace = parser.parse_args(&vec!["run".to_string(), "--strict".to_string(), "--level".to_string(), "2".to_string(), "--custom-level".to_string(), "3".to_string(), "main.sifr".to_string()]);
    assert!(parsed.get(&"cmd".to_string(), &"".to_string()) == "run".to_string());
    assert!(parsed.get_bool(&"strict".to_string(), false));
    assert!(parsed.get(&"level".to_string(), &"".to_string()) == "2".to_string());
    assert!(parsed.get(&"custom_level".to_string(), &"".to_string()) == "3".to_string());
    assert!(format!("{:?}", parsed.get_list(&"targets".to_string())) == "[\"main.sifr\"]".to_string());
}
