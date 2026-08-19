// src/main.rs
mod __sifr_project_nominals {
    pub use ::std::collections::HashMap;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eargparse_x2eArgumentSpec {
        pub name: String,
        pub dest: String,
        pub kind: String,
        pub default_value: String,
        pub nargs: String,
        pub type_name: String,
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentSpec {
        pub fn new(
            name: String,
            dest: String,
            kind: String,
            default_value: String,
            nargs: String,
            type_name: String,
        ) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
                __sifr_concat.push_str((name).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: String = {
                let mut __sifr_concat: String = String::with_capacity(dest.len() + 0usize);
                __sifr_concat.push_str((dest).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_2: String = {
                let mut __sifr_concat: String = String::with_capacity(kind.len() + 0usize);
                __sifr_concat.push_str((kind).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_3: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    default_value.len() + 0usize,
                );
                __sifr_concat.push_str((default_value).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_4: String = _normalize_nargs(&nargs);
            let __sifr_field_init_5: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    type_name.len() + 0usize,
                );
                __sifr_concat.push_str((type_name).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            Self {
                name: __sifr_field_init_0,
                dest: __sifr_field_init_1,
                kind: __sifr_field_init_2,
                default_value: __sifr_field_init_3,
                nargs: __sifr_field_init_4,
                type_name: __sifr_field_init_5,
            }
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentSpec {}
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eargparse_x2eArgumentSpec {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "ArgumentSpec(name={}, dest={}, kind={}, default_value={}, nargs={}, type_name={})",
                self.name, self.dest, self.kind, self.default_value, self.nargs, self
                .type_name
            )
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2eargparse_x2eNamespace {
        pub _str_values: Vec<(String, String)>,
        pub _bool_values: Vec<(String, bool)>,
        pub _list_values: Vec<(String, Vec<String>)>,
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eNamespace {
        pub fn new() -> Self {
            let __sifr_field_init_0: Vec<(String, String)> = vec![];
            let __sifr_field_init_1: Vec<(String, bool)> = vec![];
            let __sifr_field_init_2: Vec<(String, Vec<String>)> = vec![];
            Self {
                _str_values: __sifr_field_init_0,
                _bool_values: __sifr_field_init_1,
                _list_values: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eNamespace {
        pub fn set(&mut self, name: &String, value: &String) {
            let mut updated: Vec<(String, String)> = vec![];
            let mut replaced: bool = false;
            for (key, current) in self._str_values.clone().iter().cloned() {
                if key == *name {
                    updated.push((format!("{}{}", name, ""), format!("{}{}", value, "")));
                    replaced = true;
                } else {
                    updated.push(((key).clone(), (current).clone()));
                }
            }
            if !replaced {
                updated.push((format!("{}{}", name, ""), format!("{}{}", value, "")));
            }
            self._str_values = updated;
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eNamespace {
        pub fn set_bool(&mut self, name: &String, value: bool) {
            let mut updated: Vec<(String, bool)> = vec![];
            let mut replaced: bool = false;
            for (key, current) in self._bool_values.clone().iter().cloned() {
                if key == *name {
                    updated.push((format!("{}{}", name, ""), value));
                    replaced = true;
                } else {
                    updated.push(((key).clone(), current));
                }
            }
            if !replaced {
                updated.push((format!("{}{}", name, ""), value));
            }
            self._bool_values = updated;
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eNamespace {
        pub fn set_list(&mut self, name: &String, values: &Vec<String>) {
            let mut copied: Vec<String> = vec![];
            for value in values.iter().cloned() {
                copied.push(format!("{}{}", value, ""));
            }
            let mut updated: Vec<(String, Vec<String>)> = vec![];
            for (key, current) in self._list_values.clone().iter().cloned() {
                if key == *name {
                    continue;
                }
                updated.push(((key).clone(), (current).clone()));
            }
            updated.push((format!("{}{}", name, ""), (copied).clone()));
            self._list_values = updated;
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eNamespace {
        pub fn get(&self, name: &String, default: &String) -> String {
            for (key, value) in self._str_values.clone().iter().cloned() {
                if key == *name {
                    return {
                        let mut __sifr_concat: String = String::with_capacity(
                            value.len() + 0usize,
                        );
                        __sifr_concat.push_str((value).as_str());
                        __sifr_concat.push_str("");
                        __sifr_concat
                    };
                }
            }
            {
                let mut __sifr_concat: String = String::with_capacity(
                    default.len() + 0usize,
                );
                __sifr_concat.push_str((default).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eNamespace {
        pub fn get_bool(&self, name: &String, default: bool) -> bool {
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
                if (((normalized == "1") || (normalized == "true")) || (normalized == "yes"))
                    || (normalized == "on")
                {
                    return true;
                }
                if (((normalized == "0") || (normalized == "false")) || (normalized == "no"))
                    || (normalized == "off")
                {
                    return false;
                }
            }
            default
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eNamespace {
        pub fn get_list(&self, name: &String) -> Vec<String> {
            for (key, values) in self._list_values.clone().iter().cloned() {
                if key != *name {
                    continue;
                }
                let mut copied: Vec<String> = vec![];
                for value in values.iter().cloned() {
                    copied.push(format!("{}{}", value, ""));
                }
                return copied;
            }
            vec![]
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eNamespace {
        pub fn merge_from(&mut self, other: &__SifrStdlib_sifr_x2eargparse_x2eNamespace) {
            for (key, value) in other._str_values.clone().iter().cloned() {
                self.set(&key, &value);
            }
            for (key2, value2) in other._bool_values.clone().iter().cloned() {
                self.set_bool(&key2, value2);
            }
            for (key3, values3) in other._list_values.clone().iter().cloned() {
                self.set_list(&key3, &values3);
            }
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eNamespace {
        pub fn copy(&self) -> __SifrStdlib_sifr_x2eargparse_x2eNamespace {
            let mut copied: __SifrStdlib_sifr_x2eargparse_x2eNamespace = __SifrStdlib_sifr_x2eargparse_x2eNamespace::new();
            copied.merge_from(&self);
            copied
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub _prog: String,
        pub _specs: Vec<__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec>,
        pub _subparsers_dest: String,
        pub _subparsers: Vec<(String, Vec<__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec>)>,
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn new(prog: String) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(prog.len() + 0usize);
                __sifr_concat.push_str((prog).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: Vec<__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec> = vec![];
            let __sifr_field_init_2: String = "command".to_string();
            let __sifr_field_init_3: Vec<
                (String, Vec<__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec>),
            > = vec![];
            Self {
                _prog: __sifr_field_init_0,
                _specs: __sifr_field_init_1,
                _subparsers_dest: __sifr_field_init_2,
                _subparsers: __sifr_field_init_3,
            }
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn prog(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self._prog.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn add_subparsers(&mut self, dest: &String) {
            if (dest).as_str() != "" {
                self._subparsers_dest = {
                    let mut __sifr_concat: String = String::with_capacity(
                        dest.len() + 0usize,
                    );
                    __sifr_concat.push_str((dest).as_str());
                    __sifr_concat.push_str("");
                    __sifr_concat
                };
            }
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn add_parser(
            &mut self,
            name: &String,
            parser: __SifrStdlib_sifr_x2eargparse_x2eArgumentParser,
        ) {
            let mut specs_copy: Vec<__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec> = vec![];
            for spec in parser._specs.clone().iter().cloned() {
                specs_copy.push(spec.clone());
            }
            self._subparsers.push((format!("{}{}", name, ""), (specs_copy).clone()));
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn _append_spec(
            &mut self,
            name: &String,
            dest: &String,
            action: &String,
            default: &String,
            nargs: &String,
            type_name: &String,
        ) {
            let mut resolved_dest: String = {
                let mut __sifr_concat: String = String::with_capacity(dest.len() + 0usize);
                __sifr_concat.push_str((dest).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            if resolved_dest == "" {
                resolved_dest = _derive_dest(name);
            }
            let mut kind: String = "positional".to_string();
            if name.starts_with("-") {
                if (action).as_str() == "store_true" {
                    kind = "flag".to_string();
                } else {
                    kind = "option".to_string();
                }
            }
            let spec: __SifrStdlib_sifr_x2eargparse_x2eArgumentSpec = __SifrStdlib_sifr_x2eargparse_x2eArgumentSpec::new(
                (name).clone(),
                resolved_dest,
                kind,
                (default).clone(),
                (nargs).clone(),
                (type_name).clone(),
            );
            self._specs.push(spec.clone());
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn add_argument(
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
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn add_argument_typed(
            &mut self,
            name: &String,
            dest: &String,
            action: &String,
            default: &String,
            nargs: &String,
            type_name: &String,
        ) {
            let mut normalized_type: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    type_name.len() + 0usize,
                );
                __sifr_concat.push_str((type_name).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            if (((normalized_type != "int") && (normalized_type != "float"))
                && (normalized_type != "bool")) && (normalized_type != "str")
            {
                normalized_type = "str".to_string();
            }
            self._append_spec(name, dest, action, default, nargs, &normalized_type);
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn _find_subparser(
            &self,
            name: &String,
        ) -> Option<Vec<__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec>> {
            for (parser_name, parser_specs) in self._subparsers.clone().iter().cloned() {
                if parser_name == *name {
                    return Some(parser_specs);
                }
            }
            None
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn _coerce_token(
            &self,
            spec: &__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec,
            token: &String,
        ) -> Option<String> {
            if (spec.type_name.clone() == "int") {
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
            if (spec.type_name.clone() == "float") {
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
            if (spec.type_name.clone() == "bool") {
                return _coerce_bool(token);
            }
            Some({
                let mut __sifr_concat: String = String::with_capacity(token.len() + 0usize);
                __sifr_concat.push_str((token).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            })
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn _collect_option_values(
            &self,
            args: &Vec<String>,
            start: i64,
            spec: &__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec,
            force_positional: bool,
        ) -> (Vec<String>, i64) {
            let mut values: Vec<String> = vec![];
            let mut i: i64 = start;
            if (spec.nargs.clone() == "?") {
                if (i >= (args.len() as i64)) {
                    return (values.clone(), i);
                }
                let token_opt: Option<String> = Some(args[i as usize].clone());
                let Some(token_opt) = token_opt else {
                    return (values.clone(), i + (1_i64));
                };
                let token_one: String = _copy_token(&Some((token_opt).clone()));
                if !force_positional && _is_option_like_token(&self._specs, &token_one) {
                    return (values.clone(), i);
                }
                values.push(token_one.clone());
                return (values.clone(), i + (1_i64));
            }
            if (spec.nargs.clone() == "*") || (spec.nargs.clone() == "+") {
                while (i < (args.len() as i64)) {
                    let token_opt2: Option<String> = Some(args[i as usize].clone());
                    let Some(token_opt2) = token_opt2 else {
                        i += 1_i64;
                        continue;
                    };
                    let token_many: String = _copy_token(&Some((token_opt2).clone()));
                    if !force_positional && _is_option_like_token(&self._specs, &token_many)
                    {
                        break;
                    }
                    values.push(token_many.clone());
                    i += 1_i64;
                }
                return (values.clone(), i);
            }
            let mut exact: i64 = 1_i64;
            if _is_digit_string(&spec.nargs.clone()) {
                let __sifr_try_res: Result<(), ParseError> = (|| {
                    let parsed_count: i64 = (spec.nargs.clone())
                        .parse::<i64>()
                        .map_err(|e| ParseError {
                            message: e.to_string(),
                        })?;
                    if parsed_count > (0_i64) {
                        exact = parsed_count;
                    }
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let _e = __sifr_try_err.clone();
                    exact = 1_i64;
                }
            }
            let mut count: i64 = 0_i64;
            while (count < exact) && (i < (args.len() as i64)) {
                let token_opt3: Option<String> = Some(args[i as usize].clone());
                let Some(token_opt3) = token_opt3 else {
                    i += 1_i64;
                    continue;
                };
                let token_exact: String = _copy_token(&Some((token_opt3).clone()));
                if !force_positional && _is_option_like_token(&self._specs, &token_exact) {
                    break;
                }
                values.push(token_exact.clone());
                i += 1_i64;
                count += 1_i64;
            }
            (values.clone(), i)
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn _collect_positional_values(
            &self,
            args: &Vec<String>,
            start: i64,
            spec: &__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec,
            force_positional: bool,
        ) -> (Vec<String>, i64) {
            let mut values: Vec<String> = vec![];
            let mut i: i64 = start;
            if (i >= (args.len() as i64)) {
                return (values.clone(), i);
            }
            if (spec.nargs.clone() == "?") {
                let token_opt: Option<String> = Some(args[i as usize].clone());
                if let Some(token_opt) = token_opt {
                    let token_one: String = _copy_token(&Some((token_opt).clone()));
                    if !force_positional && _is_option_like_token(&self._specs, &token_one) {
                        return (values.clone(), i);
                    }
                    values.push(token_one.clone());
                }
                return (values.clone(), i + (1_i64));
            }
            if (spec.nargs.clone() == "*") || (spec.nargs.clone() == "+") {
                while (i < (args.len() as i64)) {
                    let token_opt2: Option<String> = Some(args[i as usize].clone());
                    let Some(token_opt2) = token_opt2 else {
                        i += 1_i64;
                        continue;
                    };
                    let token_many: String = _copy_token(&Some((token_opt2).clone()));
                    if !force_positional && _is_option_like_token(&self._specs, &token_many)
                    {
                        break;
                    }
                    values.push(token_many.clone());
                    i += 1_i64;
                }
                return (values.clone(), i);
            }
            let mut exact: i64 = 1_i64;
            if _is_digit_string(&spec.nargs.clone()) {
                let __sifr_try_res: Result<(), ParseError> = (|| {
                    let parsed_count: i64 = (spec.nargs.clone())
                        .parse::<i64>()
                        .map_err(|e| ParseError {
                            message: e.to_string(),
                        })?;
                    if parsed_count > (0_i64) {
                        exact = parsed_count;
                    }
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let _e = __sifr_try_err.clone();
                    exact = 1_i64;
                }
            }
            let mut count: i64 = 0_i64;
            while (count < exact) && (i < (args.len() as i64)) {
                let token_opt3: Option<String> = Some(args[i as usize].clone());
                if let Some(token_opt3) = token_opt3 {
                    values.push(_copy_token(&Some((token_opt3).clone())));
                    count += 1_i64;
                }
                i += 1_i64;
            }
            (values.clone(), i)
        }
    }
    impl __SifrStdlib_sifr_x2eargparse_x2eArgumentParser {
        pub fn parse_args(
            &self,
            args: &Vec<String>,
        ) -> __SifrStdlib_sifr_x2eargparse_x2eNamespace {
            let mut ns: __SifrStdlib_sifr_x2eargparse_x2eNamespace = __SifrStdlib_sifr_x2eargparse_x2eNamespace::new();
            for spec in self._specs.clone().iter().cloned() {
                if (spec.kind.clone() == "flag") {
                    ns.set_bool(&spec.dest.clone(), false);
                } else {
                    if (_nargs_is_multi(&spec.nargs.clone()) || (spec.nargs.clone() == "*"))
                        || (spec.nargs.clone() == "+")
                    {
                        ns.set_list(&spec.dest.clone(), &vec![]);
                    } else {
                        ns.set(&spec.dest.clone(), &spec.default_value.clone());
                    }
                }
            }
            if ((self._subparsers.len() as i64) > (0_i64)) && ((args.len() as i64) > (0_i64))
            {
                let first_token: Option<String> = Some(args[(0_i64) as usize].clone());
                if let Some(first_token) = first_token {
                    let command_name: String = _copy_token(&Some((first_token).clone()));
                    let subparser_specs: Option<
                        Vec<__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec>,
                    > = self._find_subparser(&command_name);
                    if let Some(subparser_specs) = subparser_specs {
                        ns.set(&self._subparsers_dest.clone(), &command_name);
                        let mut subparser: __SifrStdlib_sifr_x2eargparse_x2eArgumentParser = __SifrStdlib_sifr_x2eargparse_x2eArgumentParser::new(
                            command_name,
                        );
                        subparser._specs = subparser_specs;
                        let child_ns: __SifrStdlib_sifr_x2eargparse_x2eNamespace = subparser
                            .parse_args(
                                &({
                                    let _slice_src = &args;
                                    let _slice_len_i64 = _slice_src.len() as i64;
                                    let _slice_start_i64 = if (1_i64) < 0 {
                                        (_slice_len_i64 + (1_i64)).max(0)
                                    } else {
                                        (1_i64).min(_slice_len_i64)
                                    };
                                    let _slice_stop_i64 = if (args.len() as i64) < 0 {
                                        (_slice_len_i64 + (args.len() as i64)).max(0)
                                    } else {
                                        (args.len() as i64).min(_slice_len_i64)
                                    };
                                    Vec::from_iter(
                                        _slice_src
                                            .iter()
                                            .skip(_slice_start_i64 as usize)
                                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                                            .cloned(),
                                    )
                                }),
                            );
                        ns.merge_from(&child_ns);
                        return ns;
                    }
                }
            }
            let mut positional_specs: Vec<__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec> = vec![];
            for spec2 in self._specs.clone().iter().cloned() {
                if (spec2.kind.clone() == "positional") {
                    positional_specs.push(spec2.clone());
                }
            }
            let mut i: i64 = 0_i64;
            let mut positional_index: i64 = 0_i64;
            let mut force_positional: bool = false;
            while (i < (args.len() as i64)) {
                let token_opt: Option<String> = Some(args[i as usize].clone());
                let Some(token_opt) = token_opt else {
                    i += 1_i64;
                    continue;
                };
                let token: String = _copy_token(&Some((token_opt).clone()));
                if (token == "--") && !force_positional {
                    force_positional = true;
                    i += 1_i64;
                    continue;
                }
                if token.starts_with("-") && !force_positional {
                    let (inline_has_value, inline_name, inline_value) = _split_inline_option(
                        &token,
                    );
                    let __sifr_chars_inline_name: Vec<char> = inline_name
                        .chars()
                        .collect::<Vec<char>>();
                    let __sifr_chars_inline_value: Vec<char> = inline_value
                        .chars()
                        .collect::<Vec<char>>();
                    let mut lookup_name: String = {
                        let mut __sifr_concat: String = String::with_capacity(
                            token.len() + 0usize,
                        );
                        __sifr_concat.push_str((token).as_str());
                        __sifr_concat.push_str("");
                        __sifr_concat
                    };
                    if inline_has_value {
                        lookup_name = {
                            let mut __sifr_concat: String = String::with_capacity(
                                inline_name.len() + 0usize,
                            );
                            __sifr_concat.push_str((inline_name).as_str());
                            __sifr_concat.push_str("");
                            __sifr_concat
                        };
                    }
                    let mut handled_option: bool = false;
                    for option_spec in self._specs.clone().iter().cloned() {
                        if (option_spec.kind.clone() == "positional") {
                            continue;
                        }
                        if (option_spec.name.clone() != lookup_name) {
                            continue;
                        }
                        handled_option = true;
                        if (option_spec.kind.clone() == "flag") {
                            ns.set_bool(&option_spec.dest.clone(), true);
                            i += 1_i64;
                            break;
                        }
                        let mut values: Vec<String> = vec![];
                        if inline_has_value {
                            values = vec![inline_value.clone()];
                            i += 1_i64;
                        } else {
                            let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1) = self
                                ._collect_option_values(
                                    args,
                                    i + (1_i64),
                                    &option_spec,
                                    force_positional,
                                );
                            values = __sifr_tuple_unpack_0;
                            i = __sifr_tuple_unpack_1;
                        }
                        if (_nargs_is_multi(&option_spec.nargs.clone())
                            || (option_spec.nargs.clone() == "*"))
                            || (option_spec.nargs.clone() == "+")
                        {
                            let mut converted_values: Vec<String> = vec![];
                            for raw in values.iter().cloned() {
                                let coerced: Option<String> = self
                                    ._coerce_token(&option_spec, &raw);
                                let Some(coerced) = coerced else {
                                    continue;
                                };
                                converted_values.push(_copy_token(&Some((coerced).clone())));
                            }
                            ns.set_list(&option_spec.dest.clone(), &converted_values);
                        } else {
                            if ((values.len() as i64) > (0_i64)) {
                                let first_value: Option<String> = Some(
                                    values[(0_i64) as usize].clone(),
                                );
                                if let Some(first_value) = first_value {
                                    let token_value: String = _copy_token(
                                        &Some((first_value).clone()),
                                    );
                                    let coerced_first: Option<String> = self
                                        ._coerce_token(&option_spec, &token_value);
                                    if let Some(coerced_first) = coerced_first {
                                        let coerced_value: String = _copy_token(
                                            &Some((coerced_first).clone()),
                                        );
                                        ns.set(&option_spec.dest.clone(), &coerced_value);
                                        if (option_spec.type_name.clone() == "bool") {
                                            ns.set_bool(
                                                &option_spec.dest.clone(),
                                                (coerced_value == "true"),
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
                if (positional_index < (positional_specs.len() as i64)) {
                    let positional_spec: Option<
                        __SifrStdlib_sifr_x2eargparse_x2eArgumentSpec,
                    > = Some(positional_specs[positional_index as usize].clone());
                    if let Some(positional_spec) = positional_spec {
                        let (values2, next_i2) = self
                            ._collect_positional_values(
                                args,
                                i,
                                &positional_spec,
                                force_positional,
                            );
                        if (_nargs_is_multi(&positional_spec.nargs.clone())
                            || (positional_spec.nargs.clone() == "*"))
                            || (positional_spec.nargs.clone() == "+")
                        {
                            let mut converted_values2: Vec<String> = vec![];
                            for raw2 in values2.iter().cloned() {
                                let coerced2: Option<String> = self
                                    ._coerce_token(&positional_spec, &raw2);
                                let Some(coerced2) = coerced2 else {
                                    continue;
                                };
                                converted_values2
                                    .push(_copy_token(&Some((coerced2).clone())));
                            }
                            ns.set_list(&positional_spec.dest.clone(), &converted_values2);
                        } else {
                            if ((values2.len() as i64) > (0_i64)) {
                                let first_value2: Option<String> = Some(
                                    values2[(0_i64) as usize].clone(),
                                );
                                if let Some(first_value2) = first_value2 {
                                    let token_value2: String = _copy_token(
                                        &Some((first_value2).clone()),
                                    );
                                    let coerced_first2: Option<String> = self
                                        ._coerce_token(&positional_spec, &token_value2);
                                    if let Some(coerced_first2) = coerced_first2 {
                                        let coerced_value2: String = _copy_token(
                                            &Some((coerced_first2).clone()),
                                        );
                                        ns.set(&positional_spec.dest.clone(), &coerced_value2);
                                        if (positional_spec.type_name.clone() == "bool") {
                                            ns.set_bool(
                                                &positional_spec.dest.clone(),
                                                (coerced_value2 == "true"),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        i = next_i2;
                        positional_index += 1_i64;
                        continue;
                    }
                }
                i += 1_i64;
            }
            ns
        }
    }
    pub fn _split_inline_option(token: &String) -> (bool, String, String) {
        let __sifr_chars_token: Vec<char> = token.chars().collect::<Vec<char>>();
        let mut key: String = "".to_string();
        let mut i: i64 = 0_i64;
        while (i < (__sifr_chars_token.len() as i64)) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_token
                    .get(i as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if ch.is_some() && (ch == Some("=".to_string())) {
                let mut value: String = "".to_string();
                let mut j: i64 = i + (1_i64);
                while (j < (__sifr_chars_token.len() as i64)) {
                    let part: Option<String> = Some({
                        let Some(__indexed_char) = __sifr_chars_token
                            .get(j as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                    if let Some(part) = part {
                        value.push_str((part).as_str());
                    }
                    j += 1_i64;
                }
                return (true, key, value);
            }
            if let Some(ch) = ch {
                key.push_str((ch).as_str());
            }
            i += 1_i64;
        }
        (
            false,
            {
                let mut __sifr_concat: String = String::with_capacity(token.len() + 0usize);
                __sifr_concat.push_str((token).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            },
            "".to_string(),
        )
    }
    pub fn _is_digit_string(value: &String) -> bool {
        if (value).as_str() == "" {
            return false;
        }
        for ch in value.chars().map(|c| c.to_string()) {
            if (ch < "0".to_string()) || (ch > "9".to_string()) {
                return false;
            }
        }
        true
    }
    pub fn _normalize_nargs(nargs: &String) -> String {
        if (nargs).as_str() == "" {
            return "1".to_string();
        }
        if (((nargs).as_str() == "?") || ((nargs).as_str() == "*"))
            || ((nargs).as_str() == "+")
        {
            return {
                let mut __sifr_concat: String = String::with_capacity(nargs.len() + 0usize);
                __sifr_concat.push_str((nargs).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
        if _is_digit_string(nargs) {
            let __sifr_try_res: Result<Option<String>, ParseError> = (|| {
                let parsed: i64 = (nargs)
                    .parse::<i64>()
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                if parsed > (0_i64) {
                    return Ok(Some(format!("{}", parsed)));
                }
                Ok(None)
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
        "1".to_string()
    }
    pub fn _nargs_is_multi(nargs: &String) -> bool {
        let normalized: String = _normalize_nargs(nargs);
        if (normalized == "*") || (normalized == "+") {
            return true;
        }
        if _is_digit_string(&normalized) {
            let __sifr_try_res: Result<bool, ParseError> = (|| {
                let parsed: i64 = (normalized)
                    .parse::<i64>()
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                return Ok(parsed > (1_i64));
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
        false
    }
    pub fn _coerce_bool(raw: &String) -> Option<String> {
        let normalized: String = raw.to_lowercase();
        if (((normalized == "1") || (normalized == "true")) || (normalized == "yes"))
            || (normalized == "on")
        {
            return Some("true".to_string());
        }
        if (((normalized == "0") || (normalized == "false")) || (normalized == "no"))
            || (normalized == "off")
        {
            return Some("false".to_string());
        }
        None
    }
    pub fn _copy_token(value: &Option<String>) -> String {
        let Some(value) = value else {
            return "".to_string();
        };
        {
            let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
            __sifr_concat.push_str((value).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
    pub fn _derive_dest(name: &String) -> String {
        let __sifr_chars_name: Vec<char> = name.chars().collect::<Vec<char>>();
        if name.starts_with("--") {
            return name
                .chars()
                .skip((2_i64) as usize)
                .take(((name.chars().count() as i64) as usize) - ((2_i64) as usize))
                .collect::<String>()
                .replace('-', "_");
        }
        if name.starts_with("-") {
            return name
                .chars()
                .skip((1_i64) as usize)
                .take(((name.chars().count() as i64) as usize) - ((1_i64) as usize))
                .collect::<String>()
                .replace('-', "_");
        }
        {
            let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
            __sifr_concat.push_str((name).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
    pub fn _is_option_like_token(
        specs: &Vec<__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec>,
        token: &String,
    ) -> bool {
        if (token).as_str() == "--" {
            return true;
        }
        if token.starts_with("--") {
            return true;
        }
        let (inline_has_value, inline_name, inline_value) = _split_inline_option(token);
        let __sifr_chars_inline_name: Vec<char> = inline_name.chars().collect::<Vec<char>>();
        let __sifr_chars_inline_value: Vec<char> = inline_value
            .chars()
            .collect::<Vec<char>>();
        let _ = inline_value;
        let mut lookup_name: String = {
            let mut __sifr_concat: String = String::with_capacity(token.len() + 0usize);
            __sifr_concat.push_str((token).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        if inline_has_value {
            lookup_name = {
                let mut __sifr_concat: String = String::with_capacity(
                    inline_name.len() + 0usize,
                );
                __sifr_concat.push_str((inline_name).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
        for spec in specs.iter().cloned() {
            if (spec.kind.clone() == "positional") {
                continue;
            }
            if (spec.name.clone() == lookup_name) {
                return true;
            }
        }
        false
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2ecollections_x2eCounter<T: std::hash::Hash + Eq> {
        pub counts: HashMap<T, i64>,
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn new(source: Option<HashMap<T, i64>>, iterable: Option<Vec<T>>) -> Self {
            let mut counts: HashMap<T, i64> = HashMap::from([]);
            if let Some(source) = source {
                for key in source.keys().cloned().collect::<Vec<_>>() {
                    let value: Option<i64> = source.get(&key).copied();
                    if let Some(value) = value {
                        counts.insert(key.clone(), value);
                    }
                }
            }
            if let Some(iterable) = iterable {
                for item in iterable.iter().cloned() {
                    let value2: Option<i64> = counts.get(&item).copied();
                    if let Some(value2) = value2 {
                        counts.insert(item.clone(), value2 + (1_i64));
                    } else {
                        counts.insert(item.clone(), 1_i64);
                    }
                }
            }
            let __sifr_field_init_0: HashMap<T, i64> = counts;
            Self {
                counts: __sifr_field_init_0,
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn __iter__(&self) -> Vec<T> {
            Box::new((self.counts.keys().cloned().collect::<Vec<_>>()).into_iter())
                .collect::<Vec<_>>()
        }
    }
    impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn __getitem__(&self, key: &T) -> i64 {
            let val: Option<i64> = self.counts.get(&key).copied();
            if let Some(val) = val {
                return val;
            }
            0_i64
        }
    }
    impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn get(&self, key: &T, default: i64) -> i64 {
            let val: Option<i64> = self.counts.get(&key).copied();
            if let Some(val) = val {
                return val;
            }
            default
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn increment(&mut self, key: &T) {
            let val: Option<i64> = self.counts.get(&key).copied();
            if let Some(val) = val {
                self.counts.insert(key.clone(), val + (1_i64));
            } else {
                self.counts.insert(key.clone(), 1_i64);
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn total(&self) -> i64 {
            let mut total: i64 = 0_i64;
            for count in self.counts.values().cloned().collect::<Vec<_>>() {
                total += count;
            }
            total
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn most_common(&self, n: Option<i64>) -> Vec<(T, i64)> {
            let mut result: Vec<(T, i64)> = vec![];
            for key in self.counts.keys().cloned().collect::<Vec<_>>() {
                let count: Option<i64> = self.counts.get(&key).copied();
                if let Some(count) = count {
                    let entry: (T, i64) = (key.clone(), count);
                    result.push(entry.clone());
                }
            }
            let sz: i64 = result.len() as i64;
            let mut i: i64 = 0_i64;
            while i < sz {
                let mut j: i64 = i + (1_i64);
                while j < sz {
                    let left: Option<(T, i64)> = Some(result[i as usize].clone());
                    let right: Option<(T, i64)> = Some(result[j as usize].clone());
                    if let Some(left) = left {
                        if let Some(right) = right {
                            if ((right).1 > (left).1) {
                                {
                                    let __idx_raw = i;
                                    let __idx_norm = if __idx_raw < 0 {
                                        (result.len() as i64) + __idx_raw
                                    } else {
                                        __idx_raw
                                    };
                                    if __idx_norm >= 0 {
                                        if let Some(__elem) = result.get_mut(__idx_norm as usize) {
                                            *__elem = right.clone();
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
                                            *__elem = left.clone();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    j += 1_i64;
                }
                i += 1_i64;
            }
            let Some(n) = n else {
                return result;
            };
            if n <= (0_i64) {
                return vec![];
            }
            let mut top: Vec<(T, i64)> = vec![];
            let mut index: i64 = 0_i64;
            while index < n {
                if (index >= (result.len() as i64)) {
                    return top;
                }
                let value: Option<(T, i64)> = Some(result[index as usize].clone());
                if let Some(value) = value {
                    top.push(value.clone());
                }
                index += 1_i64;
            }
            top
        }
    }
    impl<
        T: ::std::hash::Hash + Eq + Clone + PartialOrd,
    > __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn keys(&self) -> Vec<T> {
            let mut result: Vec<T> = self.counts.keys().cloned().collect::<Vec<_>>();
            let sz: i64 = result.len() as i64;
            let mut i: i64 = 0_i64;
            while i < sz {
                let mut j: i64 = i + (1_i64);
                while j < sz {
                    let left: Option<T> = Some(result[i as usize].clone());
                    let right: Option<T> = Some(result[j as usize].clone());
                    if let Some(left) = left {
                        if let Some(right) = right {
                            if right < left {
                                {
                                    let __idx_raw = i;
                                    let __idx_norm = if __idx_raw < 0 {
                                        (result.len() as i64) + __idx_raw
                                    } else {
                                        __idx_raw
                                    };
                                    if __idx_norm >= 0 {
                                        if let Some(__elem) = result.get_mut(__idx_norm as usize) {
                                            *__elem = right.clone();
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
                                            *__elem = left.clone();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    j += 1_i64;
                }
                i += 1_i64;
            }
            result
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn items(&self) -> Vec<(T, i64)> {
            let mut result: Vec<(T, i64)> = vec![];
            for key in self.counts.keys().cloned().collect::<Vec<_>>() {
                let value: Option<i64> = self.counts.get(&key).copied();
                if let Some(value) = value {
                    let entry: (T, i64) = (key.clone(), value);
                    result.push(entry.clone());
                }
            }
            result
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn values(&self) -> Vec<i64> {
            self.counts.values().cloned().collect::<Vec<_>>()
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn copy(&self) -> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
            __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(self.counts.clone()), None)
        }
    }
    impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn clear(&mut self) {
            self.counts = HashMap::from([]);
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn update(&mut self, other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>) {
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
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn subtract(&mut self, other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>) {
            for key in other.counts.keys().cloned().collect::<Vec<_>>() {
                let other_val: Option<i64> = other.counts.get(&key).copied();
                if let Some(other_val) = other_val {
                    let existing: Option<i64> = self.counts.get(&key).copied();
                    if let Some(existing) = existing {
                        self.counts.insert(key, existing - other_val);
                    } else {
                        self.counts.insert(key, (0_i64) - other_val);
                    }
                }
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn elements(&self) -> Vec<T> {
            let mut result: Vec<T> = vec![];
            let all_keys: Vec<T> = self.counts.keys().cloned().collect::<Vec<_>>();
            let mut ki: i64 = 0_i64;
            while (ki < (all_keys.len() as i64)) {
                let key_opt: Option<T> = Some(all_keys[ki as usize].clone());
                if let Some(key_opt) = key_opt {
                    let cnt: Option<i64> = self.counts.get(&key_opt).copied();
                    if let Some(cnt) = cnt {
                        let mut i: i64 = 0_i64;
                        while i < cnt {
                            let key_copy: Option<T> = Some(all_keys[ki as usize].clone());
                            if let Some(key_copy) = key_copy {
                                result.push(key_copy.clone().clone());
                            }
                            i += 1_i64;
                        }
                    }
                }
                ki += 1_i64;
            }
            result
        }
    }
    impl<
        T: ::std::hash::Hash + Eq + Clone,
    > ::std::ops::Add<&__SifrStdlib_sifr_x2ecollections_x2eCounter<T>>
    for &__SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        type Output = __SifrStdlib_sifr_x2ecollections_x2eCounter<T>;
        fn add(
            self,
            other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>,
        ) -> Self::Output {
            let mut new_counts: HashMap<T, i64> = HashMap::from([]);
            for key in Box::new(
                (self.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
            ) {
                let a_val: Option<i64> = self.counts.get(&key).copied();
                if let Some(a_val) = a_val {
                    let b_val: Option<i64> = other.counts.get(&key).copied();
                    let mut b_count: i64 = 0_i64;
                    if let Some(b_val) = b_val {
                        b_count = b_val;
                    }
                    let total: i64 = a_val + b_count;
                    if total > (0_i64) {
                        new_counts.insert(key.clone(), total);
                    }
                }
            }
            for key2 in Box::new(
                (other.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
            ) {
                let already: Option<i64> = new_counts.get(&key2).copied();
                if already.is_none() {
                    let b_val2: Option<i64> = other.counts.get(&key2).copied();
                    if let Some(b_val2) = b_val2 {
                        if b_val2 > (0_i64) {
                            new_counts.insert(key2.clone(), b_val2);
                        }
                    }
                }
            }
            __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(new_counts), None)
        }
    }
    impl<
        T: ::std::hash::Hash + Eq + Clone,
    > ::std::ops::Sub<&__SifrStdlib_sifr_x2ecollections_x2eCounter<T>>
    for &__SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        type Output = __SifrStdlib_sifr_x2ecollections_x2eCounter<T>;
        fn sub(
            self,
            other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>,
        ) -> Self::Output {
            let mut new_counts: HashMap<T, i64> = HashMap::from([]);
            for key in Box::new(
                (self.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
            ) {
                let a_val: Option<i64> = self.counts.get(&key).copied();
                if let Some(a_val) = a_val {
                    let b_val: Option<i64> = other.counts.get(&key).copied();
                    let mut b_count: i64 = 0_i64;
                    if let Some(b_val) = b_val {
                        b_count = b_val;
                    }
                    let diff: i64 = a_val - b_count;
                    if diff > (0_i64) {
                        new_counts.insert(key.clone(), diff);
                    }
                }
            }
            __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(new_counts), None)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ParseError {
        pub message: String,
    }
    impl ParseError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ParseError {}
}
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eargparse_x2eArgumentParser;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eargparse_x2eNamespace;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecollections_x2eCounter;
use ::std::collections::HashMap;
fn _split_inline_option(token: &String) -> (bool, String, String) {
    let __sifr_chars_token: Vec<char> = token.chars().collect::<Vec<char>>();
    let mut key: String = "".to_string();
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_token.len() as i64)) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_token
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if ch.is_some() && (ch == Some("=".to_string())) {
            let mut value: String = "".to_string();
            let mut j: i64 = i + (1_i64);
            while (j < (__sifr_chars_token.len() as i64)) {
                let part: Option<String> = Some({
                    let Some(__indexed_char) = __sifr_chars_token
                        .get(j as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!(
                            "compiler-verified string index should be in range"
                        );
                    };
                    __indexed_char
                });
                if let Some(part) = part {
                    value.push_str((part).as_str());
                }
                j += 1_i64;
            }
            return (true, key, value);
        }
        if let Some(ch) = ch {
            key.push_str((ch).as_str());
        }
        i += 1_i64;
    }
    (
        false,
        {
            let mut __sifr_concat: String = String::with_capacity(token.len() + 0usize);
            __sifr_concat.push_str((token).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        },
        "".to_string(),
    )
}
fn _is_digit_string(value: &String) -> bool {
    if (value).as_str() == "" {
        return false;
    }
    for ch in value.chars().map(|c| c.to_string()) {
        if (ch < "0".to_string()) || (ch > "9".to_string()) {
            return false;
        }
    }
    true
}
fn _normalize_nargs(nargs: &String) -> String {
    if (nargs).as_str() == "" {
        return "1".to_string();
    }
    if (((nargs).as_str() == "?") || ((nargs).as_str() == "*"))
        || ((nargs).as_str() == "+")
    {
        return {
            let mut __sifr_concat: String = String::with_capacity(nargs.len() + 0usize);
            __sifr_concat.push_str((nargs).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    if _is_digit_string(nargs) {
        let __sifr_try_res: Result<Option<String>, ParseError> = (|| {
            let parsed: i64 = (nargs)
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            if parsed > (0_i64) {
                return Ok(Some(format!("{}", parsed)));
            }
            Ok(None)
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
    "1".to_string()
}
fn _nargs_is_multi(nargs: &String) -> bool {
    let normalized: String = _normalize_nargs(nargs);
    if (normalized == "*") || (normalized == "+") {
        return true;
    }
    if _is_digit_string(&normalized) {
        let __sifr_try_res: Result<bool, ParseError> = (|| {
            let parsed: i64 = (normalized)
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            return Ok(parsed > (1_i64));
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
    false
}
fn _coerce_bool(raw: &String) -> Option<String> {
    let normalized: String = raw.to_lowercase();
    if (((normalized == "1") || (normalized == "true")) || (normalized == "yes"))
        || (normalized == "on")
    {
        return Some("true".to_string());
    }
    if (((normalized == "0") || (normalized == "false")) || (normalized == "no"))
        || (normalized == "off")
    {
        return Some("false".to_string());
    }
    None
}
fn _copy_token(value: &Option<String>) -> String {
    let Some(value) = value else {
        return "".to_string();
    };
    {
        let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
        __sifr_concat.push_str((value).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn _derive_dest(name: &String) -> String {
    let __sifr_chars_name: Vec<char> = name.chars().collect::<Vec<char>>();
    if name.starts_with("--") {
        return name
            .chars()
            .skip((2_i64) as usize)
            .take(((name.chars().count() as i64) as usize) - ((2_i64) as usize))
            .collect::<String>()
            .replace('-', "_");
    }
    if name.starts_with("-") {
        return name
            .chars()
            .skip((1_i64) as usize)
            .take(((name.chars().count() as i64) as usize) - ((1_i64) as usize))
            .collect::<String>()
            .replace('-', "_");
    }
    {
        let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
        __sifr_concat.push_str((name).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn _is_option_like_token(
    specs: &Vec<__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec>,
    token: &String,
) -> bool {
    if (token).as_str() == "--" {
        return true;
    }
    if token.starts_with("--") {
        return true;
    }
    let (inline_has_value, inline_name, inline_value) = _split_inline_option(token);
    let __sifr_chars_inline_name: Vec<char> = inline_name.chars().collect::<Vec<char>>();
    let __sifr_chars_inline_value: Vec<char> = inline_value
        .chars()
        .collect::<Vec<char>>();
    let _ = inline_value;
    let mut lookup_name: String = {
        let mut __sifr_concat: String = String::with_capacity(token.len() + 0usize);
        __sifr_concat.push_str((token).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if inline_has_value {
        lookup_name = {
            let mut __sifr_concat: String = String::with_capacity(
                inline_name.len() + 0usize,
            );
            __sifr_concat.push_str((inline_name).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    for spec in specs.iter().cloned() {
        if (spec.kind.clone() == "positional") {
            continue;
        }
        if (spec.name.clone() == lookup_name) {
            return true;
        }
    }
    false
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParseError {
    message: String,
}
impl ParseError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for ParseError {}
fn main() {
    let counter: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = __SifrStdlib_sifr_x2ecollections_x2eCounter::new(
        None,
        Some(vec!["parse".to_string(), "parse".to_string(), "emit".to_string()]),
    );
    assert!((counter.get(& "parse".to_string(), 0_i64) == (2_i64)));
    let mut attempts: HashMap<String, i64> = HashMap::new();
    {
        let __elem = attempts.entry("collections_and_argparse".to_string()).or_insert(0);
        *__elem += 1_i64;
    }
    assert!(
        * attempts.entry("collections_and_argparse".to_string()).or_insert(0) == (1_i64)
    );
    let mut parser: __SifrStdlib_sifr_x2eargparse_x2eArgumentParser = __SifrStdlib_sifr_x2eargparse_x2eArgumentParser::new(
        "sifr".to_string(),
    );
    parser.add_subparsers(&"cmd".to_string());
    let mut run_parser: __SifrStdlib_sifr_x2eargparse_x2eArgumentParser = __SifrStdlib_sifr_x2eargparse_x2eArgumentParser::new(
        "run".to_string(),
    );
    run_parser
        .add_argument_typed(
            &"--strict".to_string(),
            &"strict".to_string(),
            &"store_true".to_string(),
            &"".to_string(),
            &"1".to_string(),
            &"str".to_string(),
        );
    run_parser
        .add_argument_typed(
            &"--level".to_string(),
            &"level".to_string(),
            &"store".to_string(),
            &"0".to_string(),
            &"1".to_string(),
            &"int".to_string(),
        );
    run_parser
        .add_argument_typed(
            &"--custom-level".to_string(),
            &"custom_level".to_string(),
            &"store".to_string(),
            &"0".to_string(),
            &"1".to_string(),
            &"int".to_string(),
        );
    run_parser
        .add_argument_typed(
            &"targets".to_string(),
            &"targets".to_string(),
            &"store".to_string(),
            &"".to_string(),
            &"+".to_string(),
            &"str".to_string(),
        );
    parser.add_parser(&"run".to_string(), run_parser);
    let parsed: __SifrStdlib_sifr_x2eargparse_x2eNamespace = parser
        .parse_args(
            &vec![
                "run".to_string(), "--strict".to_string(), "--level".to_string(), "2"
                .to_string(), "--custom-level".to_string(), "3".to_string(), "main.sifr"
                .to_string()
            ],
        );
    assert!((parsed.get(& "cmd".to_string(), & "".to_string()) == "run"));
    assert!(parsed.get_bool(& "strict".to_string(), false));
    assert!((parsed.get(& "level".to_string(), & "".to_string()) == "2"));
    assert!((parsed.get(& "custom_level".to_string(), & "".to_string()) == "3"));
    assert!(
        (format!("{:?}", parsed.get_list(& "targets".to_string())) == "[\"main.sifr\"]")
    );
}
