use crate::lower_module;
use sifr_python_parser::parse_module;
use sifr_type_system::Type;

#[test]
fn required_error_message_rejects_invalid_storage_and_constructors() {
    for (source, diagnostic) in [
        (
            "class Data:\n    message: str = \"default\"\nclass Bad(Data, Error):\n    pass\n",
            "inherited constructor requires a caller-supplied 'str' parameter",
        ),
        (
            "class Data:\n    message: str\n    def __init__(self):\n        self.message = \"fixed\"\nclass Bad(Data, Error):\n    pass\n",
            "inherited constructor requires a caller-supplied 'str' parameter",
        ),
        (
            "class Bad(Error):\n    def __init__(self):\n        self.message = \"fixed\"\n",
            "caller-supplied 'str' parameter",
        ),
        (
            "class Bad(Error):\n    def __init__(self, text: str = \"default\"):\n        self.message = text\n",
            "caller-supplied 'str' parameter",
        ),
        (
            "class Bad(Error):\n    message: int\n",
            "message must have type 'str'",
        ),
        (
            "class Bad(Error):\n    def __init__(self, code: int):\n        self.message = code\n",
            "message must have type 'str'",
        ),
        (
            "class Bad(Error):\n    def __init__(self, code: int):\n        self.code = code\n",
            "constructor must initialize required storage: message",
        ),
        (
            "class Bad(Error):\n    def __init__(self, message: int):\n        pass\n",
            "constructor message parameter must be a required 'str'",
        ),
        (
            "class Bad(Error):\n    message: str = \"default\"\n",
            "not a field default",
        ),
        (
            "class Bad(Error):\n    def __init__(self, message: str = \"default\"):\n        pass\n",
            "constructor message parameter must be a required 'str'",
        ),
        (
            "class Base(Error):\n    pass\nclass Bad(Base):\n    message: int\n",
            "cannot be re-annotated",
        ),
        (
            "class Base(Error):\n    pass\nclass Bad(Base):\n    def __init__(self, message: str):\n        pass\n",
            "super().__init__",
        ),
        (
            "class Bad(Error):\n    def __init__(self, flag: bool):\n        if flag:\n            self.message = \"conditional\"\n",
            "before field storage is initialized",
        ),
        (
            "class Bad(Error):\n    def __init__(self, message: str):\n        super().__init__(3)\n",
            "expected 'str'",
        ),
    ] {
        let parsed = parse_module(source).expect("valid syntax");
        let errors = match lower_module(parsed.suite()) {
            Ok(_) => panic!("invalid error contract must fail checking: {source}"),
            Err(errors) => errors,
        };
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains(diagnostic)),
            "{source}\n{errors:?}"
        );
        assert!(
            errors.iter().all(|error| error.primary_range.is_some()),
            "source diagnostics: {errors:?}"
        );
    }
}

#[test]
fn required_error_message_default_calls_require_a_string() {
    for call in ["Empty()", "Code(3)", "Code(3, 4)", "Empty(3)", "Leaf()"] {
        let source = format!(
            "class Empty(Error):\n    pass\nclass Code(Error):\n    code: int\nclass Leaf(Empty):\n    pass\ndef main():\n    value = {call}\n"
        );
        let parsed = parse_module(&source).expect("valid syntax");
        assert!(
            lower_module(parsed.suite()).is_err(),
            "{call} must fail before Rust"
        );
    }
}

#[test]
fn required_error_message_layout_is_single_typed_storage() {
    let parsed = parse_module("class Empty(Error):\n    pass\nclass Code(Error):\n    code: int\nclass Own(Error):\n    code: int\n    message: str\nclass Leaf(Code):\n    pass\nclass PythonError(Error):\n    message: str\n    kind: str\n    exception_type: str\n    traceback: str\n    context: str\n").expect("parse");
    let module = lower_module(parsed.suite())
        .expect("required message classes")
        .module;
    let layout: Vec<_> = module
        .classes
        .iter()
        .map(|class| {
            (
                class.name.as_str(),
                class
                    .fields
                    .iter()
                    .map(|(name, ty)| (name.as_str(), ty.display_name()))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    insta::assert_debug_snapshot!(layout, @r###"
    [
        (
            "Empty",
            [
                (
                    "message",
                    "str",
                ),
            ],
        ),
        (
            "Code",
            [
                (
                    "message",
                    "str",
                ),
                (
                    "code",
                    "int",
                ),
            ],
        ),
        (
            "Own",
            [
                (
                    "code",
                    "int",
                ),
                (
                    "message",
                    "str",
                ),
            ],
        ),
        (
            "Leaf",
            [],
        ),
        (
            "PythonError",
            [
                (
                    "message",
                    "str",
                ),
                (
                    "kind",
                    "str",
                ),
                (
                    "exception_type",
                    "str",
                ),
                (
                    "traceback",
                    "str",
                ),
                (
                    "context",
                    "str",
                ),
            ],
        ),
    ]
    "###);
    let leaf = module
        .classes
        .iter()
        .find(|class| class.name == "Leaf")
        .expect("leaf");
    assert!(
        matches!(&leaf.parent_type, Some(Type::Class {fields, ..}) if fields[0] == ("message".to_string(), Type::Str))
    );
    assert_eq!(
        leaf.methods
            .iter()
            .find(|method| method.name == "new")
            .expect("inherited constructor")
            .params
            .len(),
        2
    );
}
