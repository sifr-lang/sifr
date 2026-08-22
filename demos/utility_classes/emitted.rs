// src/main.rs
mod __sifr_project_nominals {
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
            let mut __sifr_successful_try_bindings: Option<(i64,)> = None;
            let __sifr_try_res: Result<Option<String>, ParseError> = (|| {
                let parsed: i64 = (nargs)
                    .parse::<i64>()
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                if parsed > (0_i64) {
                    return Ok(Some(format!("{}", parsed)));
                }
                __sifr_successful_try_bindings = Some((parsed,));
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
            let Some((parsed,)) = __sifr_successful_try_bindings else {
                unreachable!("successful try fallthrough must initialize promoted bindings");
            };
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
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
        pub message: String,
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eCycleError {}
    impl ::std::fmt::Debug for __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("CycleError").field("message", &self.message).finish()
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for __SifrStdlib_sifr_x2egraphlib_x2eCycleError {}
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub nodes: Vec<i64>,
        pub from_nodes: Vec<i64>,
        pub to_nodes: Vec<i64>,
        pub max_node: i64,
        pub _prepared: bool,
        pub _ready_order: Vec<i64>,
        pub _next_index: i64,
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub fn new() -> Self {
            let __sifr_field_init_0: Vec<i64> = vec![];
            let __sifr_field_init_1: Vec<i64> = vec![];
            let __sifr_field_init_2: Vec<i64> = vec![];
            let __sifr_field_init_3: i64 = -(1_i64);
            let __sifr_field_init_4: bool = false;
            let __sifr_field_init_5: Vec<i64> = vec![];
            let __sifr_field_init_6: i64 = 0_i64;
            Self {
                nodes: __sifr_field_init_0,
                from_nodes: __sifr_field_init_1,
                to_nodes: __sifr_field_init_2,
                max_node: __sifr_field_init_3,
                _prepared: __sifr_field_init_4,
                _ready_order: __sifr_field_init_5,
                _next_index: __sifr_field_init_6,
            }
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub fn _record_node(&mut self, node: i64) {
            if !(_contains_int(&self.nodes, node)) {
                self.nodes.push(node);
            }
            if (node > self.max_node) {
                self.max_node = node;
            }
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub fn add(&mut self, node: i64, predecessor: i64) {
            self._record_node(node);
            self._record_node(predecessor);
            self.from_nodes.push(predecessor);
            self.to_nodes.push(node);
            self._prepared = false;
            self._ready_order = vec![];
            self._next_index = 0_i64;
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub fn add_many(&mut self, node: i64, predecessors: &Vec<i64>) {
            self._record_node(node);
            if ((predecessors.len() as i64) == (0_i64)) {
                self._prepared = false;
                self._ready_order = vec![];
                self._next_index = 0_i64;
                return;
            }
            for predecessor in predecessors.iter().copied() {
                self.add(node, predecessor);
            }
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub fn _filter_order(&self, order: &Vec<i64>) -> Vec<i64> {
            let mut filtered: Vec<i64> = vec![];
            for candidate in order.iter().copied() {
                if _contains_int(&self.nodes, candidate) {
                    filtered.push(candidate);
                }
            }
            filtered
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub fn prepare(
            &mut self,
        ) -> Result<(), __SifrStdlib_sifr_x2egraphlib_x2eCycleError> {
            self._prepared = false;
            self._ready_order = vec![];
            self._next_index = 0_i64;
            if (self.max_node < (0_i64)) {
                self._prepared = true;
                return Ok(());
            }
            let mut prepare_ok: bool = false;
            let __sifr_try_res: Result<
                (Vec<i64>,),
                __SifrStdlib_sifr_x2egraphlib_x2eCycleError,
            > = (|| {
                let order: Vec<i64> = topological_sort(
                    self.max_node + (1_i64),
                    &self.from_nodes,
                    &self.to_nodes,
                )?;
                self._ready_order = self._filter_order(&order);
                self._prepared = true;
                prepare_ok = true;
                Ok((order,))
            })();
            let (order,) = match __sifr_try_res {
                Ok(__sifr_try_bindings) => __sifr_try_bindings,
                Err(__sifr_try_err) => {
                    let e = __sifr_try_err.clone();
                    self._prepared = false;
                    self._ready_order = vec![];
                    self._next_index = 0_i64;
                    return Err(
                        __SifrStdlib_sifr_x2egraphlib_x2eCycleError::new(e.message.clone()),
                    );
                }
            };
            if prepare_ok {
                return Ok(());
            }
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub fn get_ready(
            &mut self,
        ) -> Result<Vec<i64>, __SifrStdlib_sifr_x2egraphlib_x2eCycleError> {
            if !(self._prepared) {
                let __sifr_try_res: Result<
                    ((),),
                    __SifrStdlib_sifr_x2egraphlib_x2eCycleError,
                > = (|| {
                    let _prepared: () = self.prepare()?;
                    let _ = _prepared;
                    Ok((_prepared,))
                })();
                let (_prepared,) = match __sifr_try_res {
                    Ok(__sifr_try_bindings) => __sifr_try_bindings,
                    Err(__sifr_try_err) => {
                        let e = __sifr_try_err.clone();
                        return Err(
                            __SifrStdlib_sifr_x2egraphlib_x2eCycleError::new(
                                e.message.clone(),
                            ),
                        );
                    }
                };
            }
            if (self._next_index < (self._ready_order.len() as i64)) {
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
            Ok(vec![])
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub fn done(&mut self, node: i64) {
            if !(self._prepared) {
                return;
            }
            if (self._next_index >= (self._ready_order.len() as i64)) {
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
            if current.is_some() && (current == Some(node)) {
                self._next_index += 1_i64;
            }
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub fn is_active(&self) -> bool {
            if !(self._prepared) {
                return false;
            }
            (self._next_index < (self._ready_order.len() as i64))
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub fn reset(&mut self) {
            self._prepared = false;
            self._ready_order = vec![];
            self._next_index = 0_i64;
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter {
        pub fn static_order(
            &self,
        ) -> Result<Vec<i64>, __SifrStdlib_sifr_x2egraphlib_x2eCycleError> {
            if (self.max_node < (0_i64)) {
                return Ok(vec![]);
            }
            let __sifr_try_res: Result<
                Result<Vec<i64>, __SifrStdlib_sifr_x2egraphlib_x2eCycleError>,
                __SifrStdlib_sifr_x2egraphlib_x2eCycleError,
            > = (|| {
                let full_order: Vec<i64> = topological_sort(
                    self.max_node + (1_i64),
                    &self.from_nodes,
                    &self.to_nodes,
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
                    return Err(
                        __SifrStdlib_sifr_x2egraphlib_x2eCycleError::new(e.message.clone()),
                    );
                }
            }
        }
    }
    pub fn _contains_int(values: &Vec<i64>, target: i64) -> bool {
        for value in values.iter().copied() {
            if value == target {
                return true;
            }
        }
        false
    }
    pub fn topological_sort(
        num_nodes: i64,
        from_nodes: &Vec<i64>,
        to_nodes: &Vec<i64>,
    ) -> Result<Vec<i64>, __SifrStdlib_sifr_x2egraphlib_x2eCycleError> {
        let mut result: Vec<i64> = vec![];
        let mut visited: Vec<i64> = vec![];
        let mut i: i64 = 0_i64;
        while i < num_nodes {
            visited.push(0_i64);
            i += 1_i64;
        }
        let mut processed: i64 = 0_i64;
        while processed < num_nodes {
            let mut found_any: bool = false;
            let mut node: i64 = 0_i64;
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
                    if v == (0_i64) {
                        let mut has_dep: bool = false;
                        let mut j: i64 = 0_i64;
                        while (j < (to_nodes.len() as i64)) {
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
                                            if dep_v == (0_i64) {
                                                has_dep = true;
                                            }
                                        }
                                    }
                                }
                            }
                            j += 1_i64;
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
                                        *__elem = 1_i64;
                                    }
                                }
                            }
                            processed += 1_i64;
                            found_any = true;
                        }
                    }
                }
                node += 1_i64;
            }
            if !found_any {
                return Err(
                    __SifrStdlib_sifr_x2egraphlib_x2eCycleError::new(
                        "cycle detected in graph".to_string(),
                    ),
                );
            }
        }
        Ok(result)
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        pub _text: String,
        pub _value: i64,
    }
    impl __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        pub fn new(addr: String) -> Self {
            let mut normalized_text: String = {
                let mut __sifr_concat: String = String::with_capacity(addr.len() + 0usize);
                __sifr_concat.push_str((addr).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let mut normalized_value: i64 = -(1_i64);
            if is_valid_ipv4(&addr) {
                let parsed: i64 = _ip_to_int_raw(&addr);
                normalized_value = parsed;
                normalized_text = int_to_ip(parsed);
            }
            let __sifr_field_init_0: i64 = normalized_value;
            let __sifr_field_init_1: String = normalized_text;
            Self {
                _value: __sifr_field_init_0,
                _text: __sifr_field_init_1,
            }
        }
    }
    impl __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        pub fn to_str(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self._text.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        pub fn packed_int(&self) -> i64 {
            self._value
        }
    }
    impl __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        pub fn version(&self) -> i64 {
            4_i64
        }
    }
    impl __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        pub fn is_private(&self) -> bool {
            is_private(&self._text)
        }
    }
    impl __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        pub fn is_loopback(&self) -> bool {
            is_loopback(&self._text)
        }
    }
    impl __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        pub fn is_multicast(&self) -> bool {
            is_multicast(&self._text)
        }
    }
    impl __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        pub fn is_global(&self) -> bool {
            is_global(&self._text)
        }
    }
    impl __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        pub fn is_link_local(&self) -> bool {
            is_link_local(&self._text)
        }
    }
    impl __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        pub fn is_reserved(&self) -> bool {
            is_reserved(&self._text)
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "IPv4Address(_text={}, _value={})", self._text, self._value)
        }
    }
    pub fn is_valid_ipv4(addr: &String) -> bool {
        let parts: Vec<String> = addr
            .split('.')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        if (parts.len() as i64) != (4_i64) {
            return false;
        }
        for part in parts.iter().cloned() {
            let __sifr_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
            if ((__sifr_chars_part.len() as i64) == (0_i64)) {
                return false;
            }
            if ((__sifr_chars_part.len() as i64) > (3_i64)) {
                return false;
            }
            if ((__sifr_chars_part.len() as i64) > (1_i64)) {
                let first_digit: Option<String> = Some({
                    let Some(__indexed_char) = __sifr_chars_part
                        .get((0_i64) as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!("compiler-verified string index should be in range");
                    };
                    __indexed_char
                });
                if first_digit.is_some() && (first_digit == Some("0".to_string())) {
                    return false;
                }
            }
            let val: i64 = _parse_int(&part);
            if val < (0_i64) {
                return false;
            }
            if val > (255_i64) {
                return false;
            }
        }
        true
    }
    pub fn _parse_int(s: &String) -> i64 {
        let __sifr_chars_s: Vec<char> = s.chars().collect::<Vec<char>>();
        let mut result: i64 = 0_i64;
        let mut i: i64 = 0_i64;
        while (i < (__sifr_chars_s.len() as i64)) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_s
                    .get(i as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if let Some(ch) = ch {
                if ch == "0" {
                    result *= 10_i64;
                } else {
                    if ch == "1" {
                        result = (result * (10_i64)) + (1_i64);
                    } else {
                        if ch == "2" {
                            result = (result * (10_i64)) + (2_i64);
                        } else {
                            if ch == "3" {
                                result = (result * (10_i64)) + (3_i64);
                            } else {
                                if ch == "4" {
                                    result = (result * (10_i64)) + (4_i64);
                                } else {
                                    if ch == "5" {
                                        result = (result * (10_i64)) + (5_i64);
                                    } else {
                                        if ch == "6" {
                                            result = (result * (10_i64)) + (6_i64);
                                        } else {
                                            if ch == "7" {
                                                result = (result * (10_i64)) + (7_i64);
                                            } else {
                                                if ch == "8" {
                                                    result = (result * (10_i64)) + (8_i64);
                                                } else {
                                                    if ch == "9" {
                                                        result = (result * (10_i64)) + (9_i64);
                                                    } else {
                                                        return -(1_i64);
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
            i += 1_i64;
        }
        result
    }
    pub fn _ip_to_int_raw(addr: &String) -> i64 {
        let parts: Vec<String> = addr
            .split('.')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let mut result: i64 = 0_i64;
        for part in parts.iter().cloned() {
            let val: i64 = _parse_int(&part);
            result = (result * (256_i64)) + val;
        }
        result
    }
    pub fn _in_ipv4_range(value: i64, start: i64, end: i64) -> bool {
        if value < start {
            return false;
        }
        if value > end {
            return false;
        }
        true
    }
    pub fn _is_private_ipv4_value(value: i64) -> bool {
        let mut private_hit: bool = false;
        if _in_ipv4_range(value, 0_i64, 16777215_i64) {
            private_hit = true;
        } else {
            if _in_ipv4_range(value, 167772160_i64, 184549375_i64) {
                private_hit = true;
            } else {
                if _in_ipv4_range(value, 2130706432_i64, 2147483647_i64) {
                    private_hit = true;
                } else {
                    if _in_ipv4_range(value, 2851995648_i64, 2852061183_i64) {
                        private_hit = true;
                    } else {
                        if _in_ipv4_range(value, 2886729728_i64, 2887778303_i64) {
                            private_hit = true;
                        } else {
                            if _in_ipv4_range(value, 3221225472_i64, 3221225727_i64) {
                                private_hit = true;
                            } else {
                                if _in_ipv4_range(value, 3221225642_i64, 3221225643_i64) {
                                    private_hit = true;
                                } else {
                                    if _in_ipv4_range(value, 3221225984_i64, 3221226239_i64) {
                                        private_hit = true;
                                    } else {
                                        if _in_ipv4_range(value, 3232235520_i64, 3232301055_i64) {
                                            private_hit = true;
                                        } else {
                                            if _in_ipv4_range(value, 3323068416_i64, 3323199487_i64) {
                                                private_hit = true;
                                            } else {
                                                if _in_ipv4_range(value, 3325256704_i64, 3325256959_i64) {
                                                    private_hit = true;
                                                } else {
                                                    if _in_ipv4_range(value, 3405803776_i64, 3405804031_i64) {
                                                        private_hit = true;
                                                    } else {
                                                        if _in_ipv4_range(value, 4026531840_i64, 4294967295_i64) {
                                                            private_hit = true;
                                                        } else {
                                                            if value == (4294967295_i64) {
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
            if value == (3221225481_i64) {
                return false;
            }
            if value == (3221225482_i64) {
                return false;
            }
        }
        private_hit
    }
    pub fn is_private(addr: &String) -> bool {
        if !(is_valid_ipv4(addr)) {
            return false;
        }
        let val: i64 = _ip_to_int_raw(addr);
        _is_private_ipv4_value(val)
    }
    pub fn is_loopback(addr: &String) -> bool {
        if !(is_valid_ipv4(addr)) {
            return false;
        }
        let parts: Vec<String> = addr
            .split('.')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        if ((parts.len() as i64) == (4_i64)) {
            let first: Option<String> = Some(parts[(0_i64) as usize].clone());
            if let Some(first) = first {
                if first == "127" {
                    return true;
                }
            }
        }
        false
    }
    pub fn int_to_ip(val: i64) -> String {
        if (val < (0_i64)) || (val > (4294967295_i64)) {
            return "0.0.0.0".to_string();
        }
        let a: i64 = val / (16777216_i64);
        let mut rem: i64 = val % (16777216_i64);
        let b: i64 = rem / (65536_i64);
        rem %= 65536_i64;
        let c: i64 = rem / (256_i64);
        let d: i64 = rem % (256_i64);
        {
            let mut __sifr_concat: String = String::with_capacity(
                (((((0usize + 1usize) + 0usize) + 1usize) + 0usize) + 1usize) + 0usize,
            );
            __sifr_concat.push_str((format!("{}", a)).as_str());
            __sifr_concat.push('.');
            __sifr_concat.push_str((format!("{}", b)).as_str());
            __sifr_concat.push('.');
            __sifr_concat.push_str((format!("{}", c)).as_str());
            __sifr_concat.push('.');
            __sifr_concat.push_str((format!("{}", d)).as_str());
            __sifr_concat
        }
    }
    pub fn is_multicast(addr: &String) -> bool {
        if !(is_valid_ipv4(addr)) {
            return false;
        }
        let parts: Vec<String> = addr
            .split('.')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        if ((parts.len() as i64) == (4_i64)) {
            let first: Option<String> = Some(parts[(0_i64) as usize].clone());
            if let Some(first) = first {
                let val: i64 = _parse_int(&first);
                if val >= (224_i64) {
                    if val <= (239_i64) {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub fn is_global(addr: &String) -> bool {
        if !(is_valid_ipv4(addr)) {
            return false;
        }
        let val: i64 = _ip_to_int_raw(addr);
        if _in_ipv4_range(val, 1681915904_i64, 1686110207_i64) {
            return false;
        }
        !(_is_private_ipv4_value(val))
    }
    pub fn is_link_local(addr: &String) -> bool {
        if !(is_valid_ipv4(addr)) {
            return false;
        }
        let val: i64 = _ip_to_int_raw(addr);
        _in_ipv4_range(val, 2851995648_i64, 2852061183_i64)
    }
    pub fn is_reserved(addr: &String) -> bool {
        if !(is_valid_ipv4(addr)) {
            return false;
        }
        let val: i64 = _ip_to_int_raw(addr);
        _in_ipv4_range(val, 4026531840_i64, 4294967295_i64)
    }
    pub fn uuid4() -> String {
        ::sifr_stdlib::uuid::uuid4()
    }
    pub fn uuid3_text(namespace: &String, name: &String) -> String {
        ::sifr_stdlib::uuid::uuid3_text(namespace, name)
    }
    pub fn uuid5_text(namespace: &String, name: &String) -> String {
        ::sifr_stdlib::uuid::uuid5_text(namespace, name)
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub _hex: String,
    }
    impl __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub fn new(hex_str: String) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    hex_str.len() + 0usize,
                );
                __sifr_concat.push_str((hex_str).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            Self { _hex: __sifr_field_init_0 }
        }
    }
    impl __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub fn hex(&self) -> String {
            let mut result: String = "".to_string();
            let mut i: i64 = 0_i64;
            while (i < (self._hex.chars().count() as i64)) {
                let ch: Option<String> = Some({
                    let Some(__indexed_char) = self
                        ._hex
                        .clone()
                        .chars()
                        .nth(i as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!("compiler-verified string index should be in range");
                    };
                    __indexed_char
                });
                if let Some(ch) = ch {
                    if ch != "-" {
                        result.push_str((ch).as_str());
                    }
                }
                i += 1_i64;
            }
            result
        }
    }
    impl __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub fn urn(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(9usize + 0usize);
                __sifr_concat.push_str("urn:uuid:");
                __sifr_concat.push_str((self._hex.clone()).as_str());
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub fn to_str(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self._hex.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub fn version(&self) -> i64 {
            let marker: Option<String> = {
                let __sifr_index_str = &self._hex;
                let __sifr_index_i = 14_i64;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
            };
            let Some(marker) = marker else {
                return -(1_i64);
            };
            _hex_digit_value(&marker)
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2euuid_x2eUUID {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "UUID(_hex={})", self._hex)
        }
    }
    pub fn _hex_digit_value(ch: &String) -> i64 {
        if (ch).as_str() == "0" {
            return 0_i64;
        }
        if (ch).as_str() == "1" {
            return 1_i64;
        }
        if (ch).as_str() == "2" {
            return 2_i64;
        }
        if (ch).as_str() == "3" {
            return 3_i64;
        }
        if (ch).as_str() == "4" {
            return 4_i64;
        }
        if (ch).as_str() == "5" {
            return 5_i64;
        }
        if (ch).as_str() == "6" {
            return 6_i64;
        }
        if (ch).as_str() == "7" {
            return 7_i64;
        }
        if (ch).as_str() == "8" {
            return 8_i64;
        }
        if (ch).as_str() == "9" {
            return 9_i64;
        }
        if ((ch).as_str() == "a") || ((ch).as_str() == "A") {
            return 10_i64;
        }
        if ((ch).as_str() == "b") || ((ch).as_str() == "B") {
            return 11_i64;
        }
        if ((ch).as_str() == "c") || ((ch).as_str() == "C") {
            return 12_i64;
        }
        if ((ch).as_str() == "d") || ((ch).as_str() == "D") {
            return 13_i64;
        }
        if ((ch).as_str() == "e") || ((ch).as_str() == "E") {
            return 14_i64;
        }
        if ((ch).as_str() == "f") || ((ch).as_str() == "F") {
            return 15_i64;
        }
        -(1_i64)
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Error {
        pub message: String,
    }
    impl Error {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for Error {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for Error {}
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
    impl From<ParseError> for Error {
        fn from(err: ParseError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<ValueError> for Error {
        fn from(err: ValueError) -> Self {
            Self::new(err.message)
        }
    }
}
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eargparse_x2eArgumentParser;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eargparse_x2eArgumentSpec;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eargparse_x2eNamespace;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2egraphlib_x2eCycleError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eipaddress_x2eIPv4Address;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2euuid_x2eUUID;
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
        let mut __sifr_successful_try_bindings: Option<(i64,)> = None;
        let __sifr_try_res: Result<Option<String>, ParseError> = (|| {
            let parsed: i64 = (nargs)
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            if parsed > (0_i64) {
                return Ok(Some(format!("{}", parsed)));
            }
            __sifr_successful_try_bindings = Some((parsed,));
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
        let Some((parsed,)) = __sifr_successful_try_bindings else {
            unreachable!("successful try fallthrough must initialize promoted bindings");
        };
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
fn _contains_int(values: &Vec<i64>, target: i64) -> bool {
    for value in values.iter().copied() {
        if value == target {
            return true;
        }
    }
    false
}
fn topological_sort(
    num_nodes: i64,
    from_nodes: &Vec<i64>,
    to_nodes: &Vec<i64>,
) -> Result<Vec<i64>, __SifrStdlib_sifr_x2egraphlib_x2eCycleError> {
    let mut result: Vec<i64> = vec![];
    let mut visited: Vec<i64> = vec![];
    let mut i: i64 = 0_i64;
    while i < num_nodes {
        visited.push(0_i64);
        i += 1_i64;
    }
    let mut processed: i64 = 0_i64;
    while processed < num_nodes {
        let mut found_any: bool = false;
        let mut node: i64 = 0_i64;
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
                if v == (0_i64) {
                    let mut has_dep: bool = false;
                    let mut j: i64 = 0_i64;
                    while (j < (to_nodes.len() as i64)) {
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
                                        if dep_v == (0_i64) {
                                            has_dep = true;
                                        }
                                    }
                                }
                            }
                        }
                        j += 1_i64;
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
                                    *__elem = 1_i64;
                                }
                            }
                        }
                        processed += 1_i64;
                        found_any = true;
                    }
                }
            }
            node += 1_i64;
        }
        if !found_any {
            return Err(
                __SifrStdlib_sifr_x2egraphlib_x2eCycleError::new(
                    "cycle detected in graph".to_string(),
                ),
            );
        }
    }
    Ok(result)
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eipaddress_x2eAddressValueError {
    message: String,
}
impl __SifrStdlib_sifr_x2eipaddress_x2eAddressValueError {
    fn new(message: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(
                message.len() + 0usize,
            );
            __sifr_concat.push_str((message).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self {
            message: __sifr_field_init_0,
        }
    }
}
impl __SifrStdlib_sifr_x2eipaddress_x2eAddressValueError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2eipaddress_x2eAddressValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("AddressValueError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eipaddress_x2eAddressValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2eipaddress_x2eAddressValueError {}
fn is_valid_ipv4(addr: &String) -> bool {
    let parts: Vec<String> = addr
        .split('.')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (parts.len() as i64) != (4_i64) {
        return false;
    }
    for part in parts.iter().cloned() {
        let __sifr_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
        if ((__sifr_chars_part.len() as i64) == (0_i64)) {
            return false;
        }
        if ((__sifr_chars_part.len() as i64) > (3_i64)) {
            return false;
        }
        if ((__sifr_chars_part.len() as i64) > (1_i64)) {
            let first_digit: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_part
                    .get((0_i64) as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if first_digit.is_some() && (first_digit == Some("0".to_string())) {
                return false;
            }
        }
        let val: i64 = _parse_int(&part);
        if val < (0_i64) {
            return false;
        }
        if val > (255_i64) {
            return false;
        }
    }
    true
}
fn _parse_int(s: &String) -> i64 {
    let __sifr_chars_s: Vec<char> = s.chars().collect::<Vec<char>>();
    let mut result: i64 = 0_i64;
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_s.len() as i64)) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_s
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "0" {
                result *= 10_i64;
            } else {
                if ch == "1" {
                    result = (result * (10_i64)) + (1_i64);
                } else {
                    if ch == "2" {
                        result = (result * (10_i64)) + (2_i64);
                    } else {
                        if ch == "3" {
                            result = (result * (10_i64)) + (3_i64);
                        } else {
                            if ch == "4" {
                                result = (result * (10_i64)) + (4_i64);
                            } else {
                                if ch == "5" {
                                    result = (result * (10_i64)) + (5_i64);
                                } else {
                                    if ch == "6" {
                                        result = (result * (10_i64)) + (6_i64);
                                    } else {
                                        if ch == "7" {
                                            result = (result * (10_i64)) + (7_i64);
                                        } else {
                                            if ch == "8" {
                                                result = (result * (10_i64)) + (8_i64);
                                            } else {
                                                if ch == "9" {
                                                    result = (result * (10_i64)) + (9_i64);
                                                } else {
                                                    return -(1_i64);
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
        i += 1_i64;
    }
    result
}
fn _ip_to_int_raw(addr: &String) -> i64 {
    let parts: Vec<String> = addr
        .split('.')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: i64 = 0_i64;
    for part in parts.iter().cloned() {
        let val: i64 = _parse_int(&part);
        result = (result * (256_i64)) + val;
    }
    result
}
fn _in_ipv4_range(value: i64, start: i64, end: i64) -> bool {
    if value < start {
        return false;
    }
    if value > end {
        return false;
    }
    true
}
fn _is_private_ipv4_value(value: i64) -> bool {
    let mut private_hit: bool = false;
    if _in_ipv4_range(value, 0_i64, 16777215_i64) {
        private_hit = true;
    } else {
        if _in_ipv4_range(value, 167772160_i64, 184549375_i64) {
            private_hit = true;
        } else {
            if _in_ipv4_range(value, 2130706432_i64, 2147483647_i64) {
                private_hit = true;
            } else {
                if _in_ipv4_range(value, 2851995648_i64, 2852061183_i64) {
                    private_hit = true;
                } else {
                    if _in_ipv4_range(value, 2886729728_i64, 2887778303_i64) {
                        private_hit = true;
                    } else {
                        if _in_ipv4_range(value, 3221225472_i64, 3221225727_i64) {
                            private_hit = true;
                        } else {
                            if _in_ipv4_range(value, 3221225642_i64, 3221225643_i64) {
                                private_hit = true;
                            } else {
                                if _in_ipv4_range(value, 3221225984_i64, 3221226239_i64) {
                                    private_hit = true;
                                } else {
                                    if _in_ipv4_range(value, 3232235520_i64, 3232301055_i64) {
                                        private_hit = true;
                                    } else {
                                        if _in_ipv4_range(value, 3323068416_i64, 3323199487_i64) {
                                            private_hit = true;
                                        } else {
                                            if _in_ipv4_range(value, 3325256704_i64, 3325256959_i64) {
                                                private_hit = true;
                                            } else {
                                                if _in_ipv4_range(value, 3405803776_i64, 3405804031_i64) {
                                                    private_hit = true;
                                                } else {
                                                    if _in_ipv4_range(value, 4026531840_i64, 4294967295_i64) {
                                                        private_hit = true;
                                                    } else {
                                                        if value == (4294967295_i64) {
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
        if value == (3221225481_i64) {
            return false;
        }
        if value == (3221225482_i64) {
            return false;
        }
    }
    private_hit
}
fn is_private(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let val: i64 = _ip_to_int_raw(addr);
    _is_private_ipv4_value(val)
}
fn is_loopback(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let parts: Vec<String> = addr
        .split('.')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if ((parts.len() as i64) == (4_i64)) {
        let first: Option<String> = Some(parts[(0_i64) as usize].clone());
        if let Some(first) = first {
            if first == "127" {
                return true;
            }
        }
    }
    false
}
fn int_to_ip(val: i64) -> String {
    if (val < (0_i64)) || (val > (4294967295_i64)) {
        return "0.0.0.0".to_string();
    }
    let a: i64 = val / (16777216_i64);
    let mut rem: i64 = val % (16777216_i64);
    let b: i64 = rem / (65536_i64);
    rem %= 65536_i64;
    let c: i64 = rem / (256_i64);
    let d: i64 = rem % (256_i64);
    {
        let mut __sifr_concat: String = String::with_capacity(
            (((((0usize + 1usize) + 0usize) + 1usize) + 0usize) + 1usize) + 0usize,
        );
        __sifr_concat.push_str((format!("{}", a)).as_str());
        __sifr_concat.push('.');
        __sifr_concat.push_str((format!("{}", b)).as_str());
        __sifr_concat.push('.');
        __sifr_concat.push_str((format!("{}", c)).as_str());
        __sifr_concat.push('.');
        __sifr_concat.push_str((format!("{}", d)).as_str());
        __sifr_concat
    }
}
fn is_multicast(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let parts: Vec<String> = addr
        .split('.')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if ((parts.len() as i64) == (4_i64)) {
        let first: Option<String> = Some(parts[(0_i64) as usize].clone());
        if let Some(first) = first {
            let val: i64 = _parse_int(&first);
            if val >= (224_i64) {
                if val <= (239_i64) {
                    return true;
                }
            }
        }
    }
    false
}
fn is_global(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let val: i64 = _ip_to_int_raw(addr);
    if _in_ipv4_range(val, 1681915904_i64, 1686110207_i64) {
        return false;
    }
    !(_is_private_ipv4_value(val))
}
fn is_link_local(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let val: i64 = _ip_to_int_raw(addr);
    _in_ipv4_range(val, 2851995648_i64, 2852061183_i64)
}
fn is_reserved(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let val: i64 = _ip_to_int_raw(addr);
    _in_ipv4_range(val, 4026531840_i64, 4294967295_i64)
}
fn ip_address(
    addr: &String,
) -> Result<
    __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address,
    __SifrStdlib_sifr_x2eipaddress_x2eAddressValueError,
> {
    if !(is_valid_ipv4(addr)) {
        return Err(
            __SifrStdlib_sifr_x2eipaddress_x2eAddressValueError::new(
                "invalid IPv4 address".to_string(),
            ),
        );
    }
    Ok(__SifrStdlib_sifr_x2eipaddress_x2eIPv4Address::new((addr).clone()))
}
fn uuid4() -> String {
    ::sifr_stdlib::uuid::uuid4()
}
fn uuid3_text(namespace: &String, name: &String) -> String {
    ::sifr_stdlib::uuid::uuid3_text(namespace, name)
}
fn uuid5_text(namespace: &String, name: &String) -> String {
    ::sifr_stdlib::uuid::uuid5_text(namespace, name)
}
fn _to_lower_hex_char(ch: &String) -> String {
    if (ch).as_str() == "A" {
        return "a".to_string();
    }
    if (ch).as_str() == "B" {
        return "b".to_string();
    }
    if (ch).as_str() == "C" {
        return "c".to_string();
    }
    if (ch).as_str() == "D" {
        return "d".to_string();
    }
    if (ch).as_str() == "E" {
        return "e".to_string();
    }
    if (ch).as_str() == "F" {
        return "f".to_string();
    }
    {
        let mut __sifr_concat: String = String::with_capacity(ch.len() + 0usize);
        __sifr_concat.push_str((ch).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn _is_hex_char(ch: &String) -> bool {
    if (ch).as_str() == "0" {
        return true;
    }
    if (ch).as_str() == "1" {
        return true;
    }
    if (ch).as_str() == "2" {
        return true;
    }
    if (ch).as_str() == "3" {
        return true;
    }
    if (ch).as_str() == "4" {
        return true;
    }
    if (ch).as_str() == "5" {
        return true;
    }
    if (ch).as_str() == "6" {
        return true;
    }
    if (ch).as_str() == "7" {
        return true;
    }
    if (ch).as_str() == "8" {
        return true;
    }
    if (ch).as_str() == "9" {
        return true;
    }
    if (ch).as_str() == "a" {
        return true;
    }
    if (ch).as_str() == "b" {
        return true;
    }
    if (ch).as_str() == "c" {
        return true;
    }
    if (ch).as_str() == "d" {
        return true;
    }
    if (ch).as_str() == "e" {
        return true;
    }
    if (ch).as_str() == "f" {
        return true;
    }
    if (ch).as_str() == "A" {
        return true;
    }
    if (ch).as_str() == "B" {
        return true;
    }
    if (ch).as_str() == "C" {
        return true;
    }
    if (ch).as_str() == "D" {
        return true;
    }
    if (ch).as_str() == "E" {
        return true;
    }
    if (ch).as_str() == "F" {
        return true;
    }
    false
}
fn _hex_digit_value(ch: &String) -> i64 {
    if (ch).as_str() == "0" {
        return 0_i64;
    }
    if (ch).as_str() == "1" {
        return 1_i64;
    }
    if (ch).as_str() == "2" {
        return 2_i64;
    }
    if (ch).as_str() == "3" {
        return 3_i64;
    }
    if (ch).as_str() == "4" {
        return 4_i64;
    }
    if (ch).as_str() == "5" {
        return 5_i64;
    }
    if (ch).as_str() == "6" {
        return 6_i64;
    }
    if (ch).as_str() == "7" {
        return 7_i64;
    }
    if (ch).as_str() == "8" {
        return 8_i64;
    }
    if (ch).as_str() == "9" {
        return 9_i64;
    }
    if ((ch).as_str() == "a") || ((ch).as_str() == "A") {
        return 10_i64;
    }
    if ((ch).as_str() == "b") || ((ch).as_str() == "B") {
        return 11_i64;
    }
    if ((ch).as_str() == "c") || ((ch).as_str() == "C") {
        return 12_i64;
    }
    if ((ch).as_str() == "d") || ((ch).as_str() == "D") {
        return 13_i64;
    }
    if ((ch).as_str() == "e") || ((ch).as_str() == "E") {
        return 14_i64;
    }
    if ((ch).as_str() == "f") || ((ch).as_str() == "F") {
        return 15_i64;
    }
    -(1_i64)
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = __sifr_chars_value
            .get(i as usize)
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            result.push_str((ch).as_str());
        }
        i += 1_i64;
    }
    result
}
fn _starts_with(value: &String, prefix: &String) -> bool {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let __sifr_chars_prefix: Vec<char> = prefix.chars().collect::<Vec<char>>();
    if ((__sifr_chars_value.len() as i64) < (__sifr_chars_prefix.len() as i64)) {
        return false;
    }
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_prefix.len() as i64)) {
        let left: Option<String> = __sifr_chars_value
            .get(i as usize)
            .map(|c| c.to_string());
        let right: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_prefix
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if (left != right) {
            return false;
        }
        i += 1_i64;
    }
    true
}
fn _canonical_uuid_text(input_text: &String) -> Result<String, ValueError> {
    let mut normalized_input: String = {
        let mut __sifr_concat: String = String::with_capacity(input_text.len() + 0usize);
        __sifr_concat.push_str((input_text).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if _starts_with(&normalized_input, &"urn:uuid:".to_string()) {
        normalized_input = _substring(
            &normalized_input,
            9_i64,
            normalized_input.chars().count() as i64,
        );
    }
    if ((normalized_input.chars().count() as i64) >= (2_i64)) {
        let first: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 0_i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let last: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = (normalized_input.chars().count() as i64) - (1_i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if (first == Some("{".to_string())) && (last == Some("}".to_string())) {
            normalized_input = _substring(
                &normalized_input,
                1_i64,
                (normalized_input.chars().count() as i64) - (1_i64),
            );
        }
    }
    let input_len: i64 = normalized_input.chars().count() as i64;
    let mut hex_only: String = "".to_string();
    let mut i: i64 = 0_i64;
    while i < input_len {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = normalized_input
                .chars()
                .nth(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "-" {} else {
                if !(_is_hex_char(&ch)) {
                    return Err(ValueError::new("invalid UUID hex string".to_string()));
                }
                hex_only.push_str((_to_lower_hex_char(&ch)).as_str());
            }
        }
        i += 1_i64;
    }
    if ((hex_only.chars().count() as i64) != (32_i64)) {
        return Err(
            ValueError::new("UUID hex string must be 32 hex characters".to_string()),
        );
    }
    if input_len == (36_i64) {
        let h1: Option<String> = Some({
            let Some(__indexed_char) = normalized_input
                .chars()
                .nth((8_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        let h2: Option<String> = Some({
            let Some(__indexed_char) = normalized_input
                .chars()
                .nth((13_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        let h3: Option<String> = Some({
            let Some(__indexed_char) = normalized_input
                .chars()
                .nth((18_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        let h4: Option<String> = Some({
            let Some(__indexed_char) = normalized_input
                .chars()
                .nth((23_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if (((h1 != Some("-".to_string())) || (h2 != Some("-".to_string())))
            || (h3 != Some("-".to_string()))) || (h4 != Some("-".to_string()))
        {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    } else {
        if input_len != (32_i64) {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    }
    let mut canonical: String = "".to_string();
    let mut j: i64 = 0_i64;
    while (j < (hex_only.chars().count() as i64)) {
        if (((j == (8_i64)) || (j == (12_i64))) || (j == (16_i64))) || (j == (20_i64)) {
            canonical.push('-');
        }
        let part: Option<String> = Some({
            let Some(__indexed_char) = hex_only
                .chars()
                .nth(j as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(part) = part {
            canonical.push_str((part).as_str());
        }
        j += 1_i64;
    }
    Ok(canonical)
}
fn uuid4_obj() -> __SifrStdlib_sifr_x2euuid_x2eUUID {
    __SifrStdlib_sifr_x2euuid_x2eUUID::new(uuid4())
}
fn uuid_from_hex(
    hex_str: &String,
) -> Result<__SifrStdlib_sifr_x2euuid_x2eUUID, ValueError> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2euuid_x2eUUID, ValueError>,
        ValueError,
    > = (|| {
        let canonical: String = _canonical_uuid_text(hex_str)?;
        return Ok(Ok(__SifrStdlib_sifr_x2euuid_x2eUUID::new(canonical)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(ValueError::new(e.message.clone()));
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Error {
    message: String,
}
impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for Error {}
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
impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::new(err.message)
    }
}
impl From<ValueError> for Error {
    fn from(err: ValueError) -> Self {
        Self::new(err.message)
    }
}
fn main() {
    let mut parser: __SifrStdlib_sifr_x2eargparse_x2eArgumentParser = __SifrStdlib_sifr_x2eargparse_x2eArgumentParser::new(
        "e2-demo".to_string(),
    );
    parser
        .add_argument(
            &"--strict".to_string(),
            &"strict".to_string(),
            &"store_true".to_string(),
            &"".to_string(),
        );
    parser
        .add_argument(
            &"--mode".to_string(),
            &"mode".to_string(),
            &"store".to_string(),
            &"safe".to_string(),
        );
    parser
        .add_argument(
            &"entry".to_string(),
            &"entry".to_string(),
            &"store".to_string(),
            &"demo.sifr".to_string(),
        );
    let parsed: __SifrStdlib_sifr_x2eargparse_x2eNamespace = parser
        .parse_args(
            &vec![
                "--strict".to_string(), "--mode".to_string(), "parity".to_string(),
                "main.sifr".to_string()
            ],
        );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(18usize + 0usize);
        __sifr_concat.push_str("argparse.strict = "); __sifr_concat
        .push_str((format!("{}", parsed.get_bool(& "strict".to_string(), false)))
        .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("argparse.mode = "); __sifr_concat.push_str((parsed.get(&
        "mode".to_string(), & "".to_string())).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(17usize + 0usize);
        __sifr_concat.push_str("argparse.entry = "); __sifr_concat.push_str((parsed.get(&
        "entry".to_string(), & "".to_string())).as_str()); __sifr_concat }
    );
    let parsed_inline: __SifrStdlib_sifr_x2eargparse_x2eNamespace = parser
        .parse_args(
            &vec![
                "--mode=inline".to_string(), "--".to_string(), "--literal.sifr"
                .to_string()
            ],
        );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(18usize + 0usize);
        __sifr_concat.push_str("argparse.inline = "); __sifr_concat
        .push_str((parsed_inline.get(& "mode".to_string(), & "".to_string())).as_str());
        __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(19usize + 0usize);
        __sifr_concat.push_str("argparse.literal = "); __sifr_concat
        .push_str((parsed_inline.get(& "entry".to_string(), & "".to_string())).as_str());
        __sifr_concat }
    );
    let parsed_missing: __SifrStdlib_sifr_x2eargparse_x2eNamespace = parser
        .parse_args(
            &vec![
                "--mode".to_string(), "--strict".to_string(), "fallback.sifr".to_string()
            ],
        );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(24usize + 0usize);
        __sifr_concat.push_str("argparse.missing_mode = "); __sifr_concat
        .push_str((parsed_missing.get(& "mode".to_string(), & "".to_string())).as_str());
        __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(26usize + 0usize);
        __sifr_concat.push_str("argparse.missing_strict = "); __sifr_concat
        .push_str((format!("{}", parsed_missing.get_bool(& "strict".to_string(), false)))
        .as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<
        (),
        __SifrStdlib_sifr_x2eipaddress_x2eAddressValueError,
    > = (|| {
        let addr: __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address = ip_address(
            &"8.8.8.8".to_string(),
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(((18usize +
            0usize) + 8usize) + 0usize); __sifr_concat.push_str("ipaddress.value = ");
            __sifr_concat.push_str((addr.to_str()).as_str()); __sifr_concat
            .push_str(" global="); __sifr_concat.push_str((format!("{}", addr
            .is_global())).as_str()); __sifr_concat }
        );
        let link_local: __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address = ip_address(
            &"169.254.10.20".to_string(),
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(((23usize +
            0usize) + 8usize) + 0usize); __sifr_concat
            .push_str("ipaddress.link_local = "); __sifr_concat.push_str((format!("{}",
            link_local.is_link_local())).as_str()); __sifr_concat.push_str(" global=");
            __sifr_concat.push_str((format!("{}", link_local.is_global())).as_str());
            __sifr_concat }
        );
        let multicast: __SifrStdlib_sifr_x2eipaddress_x2eIPv4Address = ip_address(
            &"224.0.0.1".to_string(),
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(29usize +
            0usize); __sifr_concat.push_str("ipaddress.multicast_global = ");
            __sifr_concat.push_str((format!("{}", multicast.is_global())).as_str());
            __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(18usize +
            0usize); __sifr_concat.push_str("ipaddress.error = "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    let generated: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid4_obj();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(((15usize +
        0usize) + 6usize) + 0usize); __sifr_concat.push_str("uuid.version = ");
        __sifr_concat.push_str((format!("{}", generated.version())).as_str());
        __sifr_concat.push_str(" text="); __sifr_concat.push_str((generated.to_str())
        .as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let parsed_curly: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(
            &"{550E8400-E29B-41D4-A716-446655440000}".to_string(),
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(19usize +
            0usize); __sifr_concat.push_str("uuid.curly.parse = "); __sifr_concat
            .push_str((parsed_curly.to_str()).as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(13usize +
            0usize); __sifr_concat.push_str("uuid.error = "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
    let mut sorter: __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter = __SifrStdlib_sifr_x2egraphlib_x2eTopologicalSorter::new();
    sorter.add_many(50_i64, &vec![30_i64, 40_i64]);
    sorter.add(30_i64, 10_i64);
    sorter.add(40_i64, 10_i64);
    sorter.add_many(10_i64, &vec![]);
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2egraphlib_x2eCycleError> = (|| {
        let order: Vec<i64> = sorter.static_order()?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(17usize +
            0usize); __sifr_concat.push_str("graphlib.order = "); __sifr_concat
            .push_str((format!("{:?}", order)).as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(17usize +
            0usize); __sifr_concat.push_str("graphlib.error = "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
}
