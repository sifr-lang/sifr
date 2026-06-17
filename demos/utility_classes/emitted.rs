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

// --- stdlib: sifr.ipaddress ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AddressValueError {
    message: String,
}
impl AddressValueError {
    fn new(message: String) -> Self {
        return Self {
            message: format!("{}{}", message, "".to_string()),
        };
    }
}
impl std::fmt::Display for AddressValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}
impl std::error::Error for AddressValueError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IPv4Address {
    _text: String,
    _value: i64,
}
impl IPv4Address {
    fn new(addr: String) -> Self {
        let mut normalized_text: String = format!("{}{}", addr, "".to_string());
        let mut normalized_value: i64 = -(1 as i64);
        if is_valid_ipv4(&addr) {
            let parsed: i64 = _ip_to_int_raw(&addr);
            normalized_value = parsed;
            normalized_text = int_to_ip(parsed);
        }
        return Self {
            _value: normalized_value,
            _text: normalized_text,
        };
    }
    fn to_str(&self) -> String {
        return format!("{}{}", self._text.clone(), "".to_string());
    }
    fn packed_int(&self) -> i64 {
        return self._value;
    }
    fn version(&self) -> i64 {
        return 4 as i64;
    }
    fn is_private(&self) -> bool {
        return is_private(&self._text.clone());
    }
    fn is_loopback(&self) -> bool {
        return is_loopback(&self._text.clone());
    }
    fn is_multicast(&self) -> bool {
        return is_multicast(&self._text.clone());
    }
    fn is_global(&self) -> bool {
        return is_global(&self._text.clone());
    }
    fn is_link_local(&self) -> bool {
        return is_link_local(&self._text.clone());
    }
    fn is_reserved(&self) -> bool {
        return is_reserved(&self._text.clone());
    }
}
impl std::fmt::Display for IPv4Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "IPv4Address(_text={}, _value={})", self._text, self._value);
    }
}
fn is_valid_ipv4(addr: &String) -> bool {
    let parts: Vec<String> = addr
        .split(&".".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (parts.len() as i64) != (4 as i64) {
        return false;
    }
    for part in parts.iter().cloned() {
        if (part.len() as i64) == (0 as i64) {
            return false;
        }
        if (part.len() as i64) > (3 as i64) {
            return false;
        }
        if (part.chars().count() as i64) > (1 as i64) {
            let first_digit: Option<String> = Some({
                let Some(__indexed_char) = part.chars().nth((0 as i64) as usize) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char.to_string()
            });
            if ((first_digit != None) && (first_digit == Some("0".to_string()))) {
                return false;
            }
        }
        let val: i64 = _parse_int(&part);
        if val < (0 as i64) {
            return false;
        }
        if val > (255 as i64) {
            return false;
        }
    }
    return true;
}
fn _parse_int(s: &String) -> i64 {
    let mut result: i64 = 0 as i64;
    let mut i: i64 = 0 as i64;
    while i < (s.chars().count() as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = s.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == "0".to_string() {
                result = result * (10 as i64);
            } else {
                if ch == "1".to_string() {
                    result = (result * (10 as i64)) + (1 as i64);
                } else {
                    if ch == "2".to_string() {
                        result = (result * (10 as i64)) + (2 as i64);
                    } else {
                        if ch == "3".to_string() {
                            result = (result * (10 as i64)) + (3 as i64);
                        } else {
                            if ch == "4".to_string() {
                                result = (result * (10 as i64)) + (4 as i64);
                            } else {
                                if ch == "5".to_string() {
                                    result = (result * (10 as i64)) + (5 as i64);
                                } else {
                                    if ch == "6".to_string() {
                                        result = (result * (10 as i64)) + (6 as i64);
                                    } else {
                                        if ch == "7".to_string() {
                                            result = (result * (10 as i64)) + (7 as i64);
                                        } else {
                                            if ch == "8".to_string() {
                                                result = (result * (10 as i64)) + (8 as i64);
                                            } else {
                                                if ch == "9".to_string() {
                                                    result = (result * (10 as i64)) + (9 as i64);
                                                } else {
                                                    return -(1 as i64);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _ip_to_int_raw(addr: &String) -> i64 {
    let parts: Vec<String> = addr
        .split(&".".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: i64 = 0 as i64;
    for part in parts.iter().cloned() {
        let val: i64 = _parse_int(&part);
        result = (result * (256 as i64)) + val;
    }
    return result;
}
fn _in_ipv4_range(value: i64, start: i64, end: i64) -> bool {
    if value < start {
        return false;
    }
    if value > end {
        return false;
    }
    return true;
}
fn _is_private_ipv4_value(value: i64) -> bool {
    let mut private_hit: bool = false;
    if _in_ipv4_range(value, 0 as i64, 16777215 as i64) {
        private_hit = true;
    } else {
        if _in_ipv4_range(value, 167772160 as i64, 184549375 as i64) {
            private_hit = true;
        } else {
            if _in_ipv4_range(value, 2130706432 as i64, 2147483647 as i64) {
                private_hit = true;
            } else {
                if _in_ipv4_range(value, 2851995648 as i64, 2852061183 as i64) {
                    private_hit = true;
                } else {
                    if _in_ipv4_range(value, 2886729728 as i64, 2887778303 as i64) {
                        private_hit = true;
                    } else {
                        if _in_ipv4_range(value, 3221225472 as i64, 3221225727 as i64) {
                            private_hit = true;
                        } else {
                            if _in_ipv4_range(
                                value,
                                3221225642 as i64,
                                3221225643 as i64,
                            ) {
                                private_hit = true;
                            } else {
                                if _in_ipv4_range(
                                    value,
                                    3221225984 as i64,
                                    3221226239 as i64,
                                ) {
                                    private_hit = true;
                                } else {
                                    if _in_ipv4_range(
                                        value,
                                        3232235520 as i64,
                                        3232301055 as i64,
                                    ) {
                                        private_hit = true;
                                    } else {
                                        if _in_ipv4_range(
                                            value,
                                            3323068416 as i64,
                                            3323199487 as i64,
                                        ) {
                                            private_hit = true;
                                        } else {
                                            if _in_ipv4_range(
                                                value,
                                                3325256704 as i64,
                                                3325256959 as i64,
                                            ) {
                                                private_hit = true;
                                            } else {
                                                if _in_ipv4_range(
                                                    value,
                                                    3405803776 as i64,
                                                    3405804031 as i64,
                                                ) {
                                                    private_hit = true;
                                                } else {
                                                    if _in_ipv4_range(
                                                        value,
                                                        4026531840 as i64,
                                                        4294967295 as i64,
                                                    ) {
                                                        private_hit = true;
                                                    } else {
                                                        if value == (4294967295 as i64) {
                                                            private_hit = true;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if private_hit {
        if value == (3221225481 as i64) {
            return false;
        }
        if value == (3221225482 as i64) {
            return false;
        }
    }
    return private_hit;
}
fn is_private(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let val: i64 = _ip_to_int_raw(addr);
    return _is_private_ipv4_value(val);
}
fn is_loopback(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let parts: Vec<String> = addr
        .split(&".".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (parts.len() as i64) == (4 as i64) {
        let first: Option<String> = Some(parts[(0 as i64) as usize].clone());
        if let Some(first) = first {
            if first == "127".to_string() {
                return true;
            }
        }
    }
    return false;
}
fn int_to_ip(val: i64) -> String {
    if (val < (0 as i64)) || (val > (4294967295 as i64)) {
        return "0.0.0.0".to_string();
    }
    let a: i64 = val / (16777216 as i64);
    let mut rem: i64 = val % (16777216 as i64);
    let b: i64 = rem / (65536 as i64);
    rem = rem % (65536 as i64);
    let c: i64 = rem / (256 as i64);
    let d: i64 = rem % (256 as i64);
    return format!(
        "{}{}{}{}{}{}{}", format!("{}", a), ".".to_string(), format!("{}", b), "."
        .to_string(), format!("{}", c), ".".to_string(), format!("{}", d)
    );
}
fn is_multicast(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let parts: Vec<String> = addr
        .split(&".".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (parts.len() as i64) == (4 as i64) {
        let first: Option<String> = Some(parts[(0 as i64) as usize].clone());
        if let Some(first) = first {
            let val: i64 = _parse_int(&first);
            if val >= (224 as i64) {
                if val <= (239 as i64) {
                    return true;
                }
            }
        }
    }
    return false;
}
fn is_global(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let val: i64 = _ip_to_int_raw(addr);
    if _in_ipv4_range(val, 1681915904 as i64, 1686110207 as i64) {
        return false;
    }
    return !(_is_private_ipv4_value(val));
}
fn is_link_local(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let val: i64 = _ip_to_int_raw(addr);
    return _in_ipv4_range(val, 2851995648 as i64, 2852061183 as i64);
}
fn is_reserved(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let val: i64 = _ip_to_int_raw(addr);
    return _in_ipv4_range(val, 4026531840 as i64, 4294967295 as i64);
}
fn ip_address(addr: &String) -> Result<IPv4Address, AddressValueError> {
    if !(is_valid_ipv4(addr)) {
        return Err(AddressValueError::new("invalid IPv4 address".to_string()));
    }
    return Ok(IPv4Address::new((addr).clone()));
}

// --- stdlib: sifr.graphlib ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CycleError {
    message: String,
}
impl CycleError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}
impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}
impl std::error::Error for CycleError {}
#[derive(Debug, Clone, PartialEq)]
struct TopologicalSorter {
    nodes: Vec<i64>,
    from_nodes: Vec<i64>,
    to_nodes: Vec<i64>,
    max_node: i64,
    _prepared: bool,
    _ready_order: Vec<i64>,
    _next_index: i64,
}
impl TopologicalSorter {
    fn new() -> Self {
        return Self {
            nodes: vec![],
            from_nodes: vec![],
            to_nodes: vec![],
            max_node: -(1 as i64),
            _prepared: false,
            _ready_order: vec![],
            _next_index: 0 as i64,
        };
    }
    fn _record_node(&mut self, node: i64) {
        if !(_contains_int(&self.nodes.clone(), node)) {
            self.nodes.push(node);
        }
        if node > self.max_node {
            self.max_node = node;
        }
    }
    fn add(&mut self, node: i64, predecessor: i64) {
        self._record_node(node);
        self._record_node(predecessor);
        self.from_nodes.push(predecessor);
        self.to_nodes.push(node);
        self._prepared = false;
        self._ready_order = vec![];
        self._next_index = 0 as i64;
    }
    fn add_many(&mut self, node: i64, predecessors: &Vec<i64>) {
        self._record_node(node);
        if (predecessors.len() as i64) == (0 as i64) {
            self._prepared = false;
            self._ready_order = vec![];
            self._next_index = 0 as i64;
            return;
        }
        for predecessor in predecessors.iter().copied() {
            self.add(node, predecessor);
        }
    }
    fn _filter_order(&self, order: &Vec<i64>) -> Vec<i64> {
        let mut filtered: Vec<i64> = vec![];
        for candidate in order.iter().copied() {
            if _contains_int(&self.nodes.clone(), candidate) {
                filtered.push(candidate);
            }
        }
        return filtered;
    }
    fn prepare(&mut self) -> Result<(), CycleError> {
        self._prepared = false;
        self._ready_order = vec![];
        self._next_index = 0 as i64;
        if self.max_node < (0 as i64) {
            self._prepared = true;
            return Ok(());
        }
        let mut prepare_ok: bool = false;
        let __sifr_try_res: Result<(), CycleError> = (|| {
            let order: Vec<i64> = topological_sort(
                self.max_node + (1 as i64),
                &self.from_nodes.clone(),
                &self.to_nodes.clone(),
            )?;
            self._ready_order = self._filter_order(&order);
            self._prepared = true;
            prepare_ok = true;
            return Ok(());
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            self._prepared = false;
            self._ready_order = vec![];
            self._next_index = 0 as i64;
            return Err(CycleError::new(e.message));
        }
        if prepare_ok {
            return Ok(());
        }
        return Ok(());
    }
    fn get_ready(&mut self) -> Result<Vec<i64>, CycleError> {
        if !(self._prepared) {
            let __sifr_try_res: Result<(), CycleError> = (|| {
                let _prepared: () = self.prepare()?;
                let _: () = _prepared;
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(CycleError::new(e.message));
            }
        }
        if self._next_index < (self._ready_order.clone().len() as i64) {
            let current: Option<i64> = {
                let __sifr_index_list = &self._ready_order;
                let __sifr_index_i = self._next_index;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).copied()
            };
            if let Some(current) = current {
                return Ok(vec![current]);
            }
        }
        return Ok(vec![]);
    }
    fn done(&mut self, node: i64) {
        if !(self._prepared) {
            return;
        }
        if self._next_index >= (self._ready_order.clone().len() as i64) {
            return;
        }
        let current: Option<i64> = {
            let __sifr_index_list = &self._ready_order;
            let __sifr_index_i = self._next_index;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if ((current != None) && (current == Some(node))) {
            self._next_index = self._next_index + (1 as i64);
        }
    }
    fn is_active(&self) -> bool {
        if !(self._prepared) {
            return false;
        }
        return self._next_index < (self._ready_order.clone().len() as i64);
    }
    fn reset(&mut self) {
        self._prepared = false;
        self._ready_order = vec![];
        self._next_index = 0 as i64;
    }
    fn static_order(&self) -> Result<Vec<i64>, CycleError> {
        if self.max_node < (0 as i64) {
            return Ok(vec![]);
        }
        let __sifr_try_res: Result<Result<Vec<i64>, CycleError>, CycleError> = (|| {
            let full_order: Vec<i64> = topological_sort(
                self.max_node + (1 as i64),
                &self.from_nodes.clone(),
                &self.to_nodes.clone(),
            )?;
            return Ok(Ok(self._filter_order(&full_order)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(CycleError::new(e.message));
            }
        }
    }
}
fn _contains_int(values: &Vec<i64>, target: i64) -> bool {
    for value in values.iter().copied() {
        if value == target {
            return true;
        }
    }
    return false;
}
fn topological_sort(
    num_nodes: i64,
    from_nodes: &Vec<i64>,
    to_nodes: &Vec<i64>,
) -> Result<Vec<i64>, CycleError> {
    let mut result: Vec<i64> = vec![];
    let mut visited: Vec<i64> = vec![];
    let mut i: i64 = 0 as i64;
    while i < num_nodes {
        visited.push(0 as i64);
        i = i + (1 as i64);
    }
    let mut processed: i64 = 0 as i64;
    while processed < num_nodes {
        let mut found_any: bool = false;
        let mut node: i64 = 0 as i64;
        while node < num_nodes {
            let v: Option<i64> = {
                let __sifr_index_list = &visited;
                let __sifr_index_i = node;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).copied()
            };
            if let Some(v) = v {
                if v == (0 as i64) {
                    let mut has_dep: bool = false;
                    let mut j: i64 = 0 as i64;
                    while j < (to_nodes.len() as i64) {
                        let to_val: Option<i64> = Some(to_nodes[j as usize]);
                        let from_val: Option<i64> = {
                            let __sifr_index_list = &from_nodes;
                            let __sifr_index_i = j;
                            let __sifr_index_norm = if __sifr_index_i < 0 {
                                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                            } else {
                                __sifr_index_i as usize
                            };
                            __sifr_index_list.get(__sifr_index_norm).copied()
                        };
                        if let Some(to_val) = to_val {
                            if let Some(from_val) = from_val {
                                if to_val == node {
                                    let dep_v: Option<i64> = {
                                        let __sifr_index_list = &visited;
                                        let __sifr_index_i = from_val;
                                        let __sifr_index_norm = if __sifr_index_i < 0 {
                                            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                                        } else {
                                            __sifr_index_i as usize
                                        };
                                        __sifr_index_list.get(__sifr_index_norm).copied()
                                    };
                                    if let Some(dep_v) = dep_v {
                                        if dep_v == (0 as i64) {
                                            has_dep = true;
                                        }
                                    }
                                }
                            }
                        }
                        j = j + (1 as i64);
                    }
                    if !has_dep {
                        result.push(node);
                        {
                            let __idx_raw = node;
                            let __idx_norm = if __idx_raw < 0 {
                                (visited.len() as i64) + __idx_raw
                            } else {
                                __idx_raw
                            };
                            if __idx_norm >= 0 {
                                if let Some(__elem) = visited.get_mut(__idx_norm as usize) {
                                    *__elem = 1 as i64;
                                }
                            }
                        }
                        processed = processed + (1 as i64);
                        found_any = true;
                    }
                }
            }
            node = node + (1 as i64);
        }
        if !found_any {
            return Err(CycleError::new("cycle detected in graph".to_string()));
        }
    }
    return Ok(result);
}

// --- stdlib: sifr.uuid ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UUID {
    _hex: String,
}
impl UUID {
    fn new(hex_str: String) -> Self {
        return Self {
            _hex: format!("{}{}", hex_str, "".to_string()),
        };
    }
    fn hex(&self) -> String {
        let mut result: String = "".to_string();
        let mut i: i64 = 0 as i64;
        while i < (self._hex.clone().chars().count() as i64) {
            let ch: Option<String> = {
                let __sifr_index_str = &self._hex;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
            };
            if let Some(ch) = ch {
                if ch != "-".to_string() {
                    result = format!("{}{}", result, ch);
                }
            }
            i = i + (1 as i64);
        }
        return result;
    }
    fn urn(&self) -> String {
        return format!("{}{}", "urn:uuid:".to_string(), self._hex.clone());
    }
    fn to_str(&self) -> String {
        return format!("{}{}", self._hex.clone(), "".to_string());
    }
    fn version(&self) -> i64 {
        let marker: Option<String> = {
            let __sifr_index_str = &self._hex;
            let __sifr_index_i = 14 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let Some(marker) = marker else {
            return -(1 as i64);
        };
        return _hex_digit_value(&marker);
    }
}
impl std::fmt::Display for UUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "UUID(_hex={})", self._hex);
    }
}
fn _to_lower_hex_char(ch: &String) -> String {
    if ch.clone() == "A".to_string() {
        return "a".to_string();
    }
    if ch.clone() == "B".to_string() {
        return "b".to_string();
    }
    if ch.clone() == "C".to_string() {
        return "c".to_string();
    }
    if ch.clone() == "D".to_string() {
        return "d".to_string();
    }
    if ch.clone() == "E".to_string() {
        return "e".to_string();
    }
    if ch.clone() == "F".to_string() {
        return "f".to_string();
    }
    return format!("{}{}", ch, "".to_string());
}
fn _is_hex_char(ch: &String) -> bool {
    if ch.clone() == "0".to_string() {
        return true;
    }
    if ch.clone() == "1".to_string() {
        return true;
    }
    if ch.clone() == "2".to_string() {
        return true;
    }
    if ch.clone() == "3".to_string() {
        return true;
    }
    if ch.clone() == "4".to_string() {
        return true;
    }
    if ch.clone() == "5".to_string() {
        return true;
    }
    if ch.clone() == "6".to_string() {
        return true;
    }
    if ch.clone() == "7".to_string() {
        return true;
    }
    if ch.clone() == "8".to_string() {
        return true;
    }
    if ch.clone() == "9".to_string() {
        return true;
    }
    if ch.clone() == "a".to_string() {
        return true;
    }
    if ch.clone() == "b".to_string() {
        return true;
    }
    if ch.clone() == "c".to_string() {
        return true;
    }
    if ch.clone() == "d".to_string() {
        return true;
    }
    if ch.clone() == "e".to_string() {
        return true;
    }
    if ch.clone() == "f".to_string() {
        return true;
    }
    if ch.clone() == "A".to_string() {
        return true;
    }
    if ch.clone() == "B".to_string() {
        return true;
    }
    if ch.clone() == "C".to_string() {
        return true;
    }
    if ch.clone() == "D".to_string() {
        return true;
    }
    if ch.clone() == "E".to_string() {
        return true;
    }
    if ch.clone() == "F".to_string() {
        return true;
    }
    return false;
}
fn _hex_digit_value(ch: &String) -> i64 {
    if ch.clone() == "0".to_string() {
        return 0 as i64;
    }
    if ch.clone() == "1".to_string() {
        return 1 as i64;
    }
    if ch.clone() == "2".to_string() {
        return 2 as i64;
    }
    if ch.clone() == "3".to_string() {
        return 3 as i64;
    }
    if ch.clone() == "4".to_string() {
        return 4 as i64;
    }
    if ch.clone() == "5".to_string() {
        return 5 as i64;
    }
    if ch.clone() == "6".to_string() {
        return 6 as i64;
    }
    if ch.clone() == "7".to_string() {
        return 7 as i64;
    }
    if ch.clone() == "8".to_string() {
        return 8 as i64;
    }
    if ch.clone() == "9".to_string() {
        return 9 as i64;
    }
    if ((ch.clone() == "a".to_string()) || (ch.clone() == "A".to_string())) {
        return 10 as i64;
    }
    if ((ch.clone() == "b".to_string()) || (ch.clone() == "B".to_string())) {
        return 11 as i64;
    }
    if ((ch.clone() == "c".to_string()) || (ch.clone() == "C".to_string())) {
        return 12 as i64;
    }
    if ((ch.clone() == "d".to_string()) || (ch.clone() == "D".to_string())) {
        return 13 as i64;
    }
    if ((ch.clone() == "e".to_string()) || (ch.clone() == "E".to_string())) {
        return 14 as i64;
    }
    if ((ch.clone() == "f".to_string()) || (ch.clone() == "F".to_string())) {
        return 15 as i64;
    }
    return -(1 as i64);
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = {
            let __sifr_index_str = &value;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if let Some(ch) = ch {
            result = format!("{}{}", result, ch);
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _starts_with(value: &String, prefix: &String) -> bool {
    if (value.len() as i64) < (prefix.len() as i64) {
        return false;
    }
    let mut i: i64 = 0 as i64;
    while i < (prefix.chars().count() as i64) {
        let left: Option<String> = {
            let __sifr_index_str = &value;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let right: Option<String> = Some({
            let Some(__indexed_char) = prefix.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if left != right {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}
fn _canonical_uuid_text(input_text: &String) -> Result<String, ValueError> {
    let mut normalized_input: String = format!("{}{}", input_text, "".to_string());
    if _starts_with(&normalized_input, &"urn:uuid:".to_string()) {
        normalized_input = _substring(
            &normalized_input,
            9 as i64,
            normalized_input.chars().count() as i64,
        );
    }
    if (normalized_input.chars().count() as i64) >= (2 as i64) {
        let first: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 0 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let last: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = (normalized_input.chars().count() as i64) - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if ((first == Some("{".to_string())) && (last == Some("}".to_string()))) {
            normalized_input = _substring(
                &normalized_input,
                1 as i64,
                (normalized_input.chars().count() as i64) - (1 as i64),
            );
        }
    }
    let input_len: i64 = normalized_input.chars().count() as i64;
    let mut hex_only: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < input_len {
        let ch_opt: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "-".to_string() {} else {
                if !(_is_hex_char(&ch)) {
                    return Err(ValueError::new("invalid UUID hex string".to_string()));
                }
                hex_only = format!("{}{}", hex_only, _to_lower_hex_char(& ch));
            }
        }
        i = i + (1 as i64);
    }
    if (hex_only.chars().count() as i64) != (32 as i64) {
        return Err(
            ValueError::new("UUID hex string must be 32 hex characters".to_string()),
        );
    }
    if input_len == (36 as i64) {
        let h1: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 8 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let h2: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 13 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let h3: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 18 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let h4: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 23 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if ((((h1 != Some("-".to_string())) || (h2 != Some("-".to_string())))
            || (h3 != Some("-".to_string()))) || (h4 != Some("-".to_string())))
        {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    } else {
        if input_len != (32 as i64) {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    }
    let mut canonical: String = "".to_string();
    let mut j: i64 = 0 as i64;
    while j < (hex_only.chars().count() as i64) {
        if (((j == (8 as i64)) || (j == (12 as i64))) || (j == (16 as i64)))
            || (j == (20 as i64))
        {
            canonical = format!("{}{}", canonical, "-".to_string());
        }
        let part: Option<String> = Some({
            let Some(__indexed_char) = hex_only.chars().nth(j as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(part) = part {
            canonical = format!("{}{}", canonical, part);
        }
        j = j + (1 as i64);
    }
    return Ok(canonical);
}
fn uuid4_obj() -> UUID {
    return UUID::new({
        let seg1 = rand::random::<u32>();
        let seg2 = rand::random::<u16>();
        let seg3 = (rand::random::<u16>() & 4095) | 16384;
        let seg4 = (rand::random::<u16>() & 16383) | 32768;
        let seg5_hi = rand::random::<u32>();
        let seg5_lo = rand::random::<u16>();
        let seg5 = ((seg5_hi as u64) << 16) | (seg5_lo as u64);
        format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", seg1, seg2, seg3, seg4, seg5)
    });
}
fn uuid_from_hex(hex_str: &String) -> Result<UUID, ValueError> {
    let __sifr_try_res: Result<Result<UUID, ValueError>, ValueError> = (|| {
        let canonical: String = _canonical_uuid_text(hex_str)?;
        return Ok(Ok(UUID::new(canonical)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(ValueError::new(e.message));
        }
    }
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

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ValueError {
}

fn main() {
    let mut parser: ArgumentParser = ArgumentParser::new("e2-demo".to_string());
    parser.add_argument(&"--strict".to_string(), &"strict".to_string(), &"store_true".to_string(), &"".to_string());
    parser.add_argument(&"--mode".to_string(), &"mode".to_string(), &"store".to_string(), &"safe".to_string());
    parser.add_argument(&"entry".to_string(), &"entry".to_string(), &"store".to_string(), &"demo.sifr".to_string());
    let mut parsed: Namespace = parser.parse_args(&vec!["--strict".to_string(), "--mode".to_string(), "parity".to_string(), "main.sifr".to_string()]);
    println!("{}", format!("{}{}", "argparse.strict = ".to_string(), format!("{}", parsed.get_bool(&"strict".to_string(), false))));
    println!("{}", format!("{}{}", "argparse.mode = ".to_string(), parsed.get(&"mode".to_string(), &"".to_string())));
    println!("{}", format!("{}{}", "argparse.entry = ".to_string(), parsed.get(&"entry".to_string(), &"".to_string())));
    let mut parsed_inline: Namespace = parser.parse_args(&vec!["--mode=inline".to_string(), "--".to_string(), "--literal.sifr".to_string()]);
    println!("{}", format!("{}{}", "argparse.inline = ".to_string(), parsed_inline.get(&"mode".to_string(), &"".to_string())));
    println!("{}", format!("{}{}", "argparse.literal = ".to_string(), parsed_inline.get(&"entry".to_string(), &"".to_string())));
    let mut parsed_missing: Namespace = parser.parse_args(&vec!["--mode".to_string(), "--strict".to_string(), "fallback.sifr".to_string()]);
    println!("{}", format!("{}{}", "argparse.missing_mode = ".to_string(), parsed_missing.get(&"mode".to_string(), &"".to_string())));
    println!("{}", format!("{}{}", "argparse.missing_strict = ".to_string(), format!("{}", parsed_missing.get_bool(&"strict".to_string(), false))));
    let __sifr_try_res: Result<(), AddressValueError> = (|| {
    let mut addr: IPv4Address = ip_address(&"8.8.8.8".to_string())?;
    println!("{}", format!("{}{}{}{}", "ipaddress.value = ".to_string(), addr.to_str(), " global=".to_string(), format!("{}", addr.is_global())));
    let mut link_local: IPv4Address = ip_address(&"169.254.10.20".to_string())?;
    println!("{}", format!("{}{}{}{}", "ipaddress.link_local = ".to_string(), format!("{}", link_local.is_link_local()), " global=".to_string(), format!("{}", link_local.is_global())));
    let mut multicast: IPv4Address = ip_address(&"224.0.0.1".to_string())?;
    println!("{}", format!("{}{}", "ipaddress.multicast_global = ".to_string(), format!("{}", multicast.is_global())));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "ipaddress.error = ".to_string(), e.message));
    }
    let mut generated: UUID = uuid4_obj();
    println!("{}", format!("{}{}{}{}", "uuid.version = ".to_string(), format!("{}", generated.version()), " text=".to_string(), generated.to_str()));
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let mut parsed_curly: UUID = uuid_from_hex(&"{550E8400-E29B-41D4-A716-446655440000}".to_string())?;
    println!("{}", format!("{}{}", "uuid.curly.parse = ".to_string(), parsed_curly.to_str()));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "uuid.error = ".to_string(), e.message));
    }
    let mut sorter: TopologicalSorter = TopologicalSorter::new();
    sorter.add_many(50 as i64, &vec![30 as i64, 40 as i64]);
    sorter.add(30 as i64, 10 as i64);
    sorter.add(40 as i64, 10 as i64);
    sorter.add_many(10 as i64, &vec![]);
    let __sifr_try_res: Result<(), CycleError> = (|| {
    let order: Vec<i64> = sorter.static_order()?;
    println!("{}", format!("{}{}", "graphlib.order = ".to_string(), format!("{:?}", order)));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "graphlib.error = ".to_string(), e.message));
    }
}
