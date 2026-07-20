use super::Type;

const PYTHON_ERROR_FIELDS: [&str; 5] =
    ["message", "kind", "exception_type", "traceback", "context"];
const PYTHON_OBJECT_IDENTITY: &str = "_sifr.python.Object";
const PYTHON_RESOURCE_IDENTITY: &str = "_sifr.python.ResourceIdentity";

impl Type {
    /// Whether this type has the exact source-level contract required to receive
    /// errors produced by the Python interop runtime.
    #[must_use]
    pub fn is_python_error_contract(&self) -> bool {
        let Self::Class {
            name,
            type_args,
            fields,
            ..
        } = self.resolve_alias()
        else {
            return false;
        };
        name == "PythonError"
            && type_args.is_empty()
            && fields.len() == PYTHON_ERROR_FIELDS.len()
            && PYTHON_ERROR_FIELDS.iter().all(|expected| {
                fields
                    .iter()
                    .any(|(name, ty)| name == expected && ty.resolve_alias() == &Self::Str)
            })
            && fields.iter().all(|(name, ty)| {
                PYTHON_ERROR_FIELDS.contains(&name.as_str()) && ty.resolve_alias() == &Self::Str
            })
    }

    /// Whether this is the sealed raw Python object exported by `sifr.python`.
    #[must_use]
    pub fn is_python_object_contract(&self) -> bool {
        matches!(
            self.resolve_alias(),
            Self::Class {
                identity: Some(identity),
                name,
                ..
            } if name == "Object" && identity == PYTHON_OBJECT_IDENTITY
        )
    }

    /// Whether this is the sealed resource owner shared by the Python protocol
    /// bridges.
    #[must_use]
    pub fn is_python_resource_identity_contract(&self) -> bool {
        matches!(
            self.resolve_alias(),
            Self::Class {
                identity: Some(identity),
                name,
                ..
            } if name == "ResourceIdentity" && identity == PYTHON_RESOURCE_IDENTITY
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Type;

    fn python_error(fields: Vec<(&str, Type)>) -> Type {
        Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "PythonError".to_string(),
            fields: fields
                .into_iter()
                .map(|(name, ty)| (name.to_string(), ty))
                .collect(),
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        }
    }

    #[test]
    fn python_error_contract_requires_the_runtime_shape() {
        let canonical = python_error(vec![
            ("message", Type::Str),
            ("kind", Type::Str),
            ("exception_type", Type::Str),
            ("traceback", Type::Str),
            ("context", Type::Str),
        ]);
        assert!(canonical.is_python_error_contract());

        let wrong_field = python_error(vec![
            ("message", Type::Str),
            ("kind", Type::Str),
            ("exception_type", Type::Str),
            ("traceback", Type::Str),
            ("context", Type::Int),
        ]);
        assert!(!wrong_field.is_python_error_contract());

        let extra_field = python_error(vec![
            ("message", Type::Str),
            ("kind", Type::Str),
            ("exception_type", Type::Str),
            ("traceback", Type::Str),
            ("context", Type::Str),
            ("code", Type::Int),
        ]);
        assert!(!extra_field.is_python_error_contract());

        let duplicate_field = python_error(vec![
            ("message", Type::Str),
            ("message", Type::Str),
            ("kind", Type::Str),
            ("exception_type", Type::Str),
            ("traceback", Type::Str),
            ("context", Type::Str),
        ]);
        assert!(!duplicate_field.is_python_error_contract());
    }

    #[test]
    fn python_object_contract_requires_the_canonical_import_identity() {
        let object = |identity: Option<&str>| Type::Class {
            identity: identity.map(str::to_string),
            type_args: Vec::new(),
            name: "Object".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: Some("NonSend".to_string()),
        };
        assert!(object(Some("_sifr.python.Object")).is_python_object_contract());
        assert!(!object(None).is_python_object_contract());
        assert!(!object(Some("local.Object")).is_python_object_contract());
        assert_eq!(
            object(Some("_sifr.python.Object")).rust_type(),
            "::sifr_runtime::interop::Handle<::sifr_runtime::python::ForeignObject>"
        );
        let canonical = object(Some("_sifr.python.Object")).union_variant_name();
        let local = object(None).union_variant_name();
        assert!(canonical.starts_with("__SifrUnionVariant_"));
        assert_ne!(canonical, local);
        assert_eq!(object(None).rust_type(), "Object");
    }

    #[test]
    fn python_resource_identity_contract_requires_the_canonical_import_identity() {
        let resource = |identity: Option<&str>| Type::Class {
            identity: identity.map(str::to_string),
            type_args: Vec::new(),
            name: "ResourceIdentity".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: Some("NonSend".to_string()),
        };
        let canonical = resource(Some("_sifr.python.ResourceIdentity"));
        assert!(canonical.is_python_resource_identity_contract());
        assert_eq!(
            canonical.rust_type(),
            "::sifr_runtime::interop::Handle<::sifr_runtime::python::PythonResourceIdentity>"
        );
        assert!(!resource(None).is_python_resource_identity_contract());
        assert!(!resource(Some("local.ResourceIdentity")).is_python_resource_identity_contract());
    }

    #[test]
    fn canonical_file_handles_use_compiler_owned_rust_names() {
        let handle = |name: &str, identity: &str| Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };

        assert_eq!(
            handle("NativeFileHandle", "_sifr.fs.NativeFileHandle").rust_type(),
            "__SifrIoNativeFileHandle"
        );
        assert_eq!(
            handle("FileHandle", "sifr.io.FileHandle").rust_type(),
            "__SifrIoFileHandle"
        );
        assert_eq!(
            handle("BinaryFileHandle", "sifr.io.BinaryFileHandle").rust_type(),
            "__SifrIoBinaryFileHandle"
        );
        assert_eq!(
            handle("TextFileHandle", "sifr.io.TextFileHandle").rust_type(),
            "__SifrIoTextFileHandle"
        );
        assert_eq!(
            handle("FileHandle", "local.FileHandle").rust_type(),
            "FileHandle"
        );
        let source_internal = handle("__SifrIoFileHandle", "local.__SifrIoFileHandle");
        assert_eq!(
            source_internal.rust_type(),
            crate::source_class_rust_name("__SifrIoFileHandle")
        );
        assert_ne!(
            source_internal.rust_type(),
            handle("FileHandle", "sifr.io.FileHandle").rust_type()
        );
    }
}
