use super::Type;

impl Type {
    /// Whether this is the compiler-provided root `Error` declaration.
    ///
    /// Imported and source-defined classes with the same basename are nominally
    /// distinct and must remain ordinary, specific exception handlers.
    #[must_use]
    pub fn is_builtin_error_base(&self) -> bool {
        matches!(
            self.resolve_alias(),
            Self::Class {
                identity: None,
                type_args,
                name,
                fields,
                parent_class: None,
                ..
            } if type_args.is_empty()
                && name == "Error"
                && fields.len() == 1
                && fields[0].0 == "message"
                && matches!(fields[0].1.resolve_alias(), Self::Str)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Type;

    fn error(identity: Option<&str>, parent_class: Option<&str>) -> Type {
        Type::Class {
            identity: identity.map(str::to_string),
            type_args: Vec::new(),
            name: "Error".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: parent_class.map(str::to_string),
        }
    }

    #[test]
    fn builtin_error_base_rejects_same_basename_declarations() {
        assert!(error(None, None).is_builtin_error_base());
        assert!(!error(Some("sifr.csv.Error"), Some("Error")).is_builtin_error_base());
        assert!(!error(None, Some("Error")).is_builtin_error_base());
    }
}
