use crate::test_support::{TestExpectErr as _, TestUnwrap as _};

use super::bridge_inventory_tests::BridgeFixture;
use super::discover_python_bridge_inventory;
use sifr_diagnostics::DiagnosticCode;

#[test]
fn dynamic_import_calls_and_aliases_are_rejected() {
    for (case, source, call) in [
        ("builtin", "value = __import__('json')\n", "__import__"),
        (
            "module_alias",
            "import importlib as loader\nvalue = loader.import_module('json')\n",
            "loader.import_module",
        ),
        (
            "function_alias",
            "from importlib import import_module as load\nvalue = load('json')\n",
            "load",
        ),
        (
            "module_assignment",
            "import importlib\nloader = importlib\nvalue = loader.import_module('json')\n",
            "loader.import_module",
        ),
        (
            "function_assignment",
            "import importlib\nload = importlib.import_module\nvalue = load('json')\n",
            "load",
        ),
        (
            "tuple_assignment",
            "import importlib\nloader, load = importlib, importlib.import_module\nvalue = load('json')\n",
            "load",
        ),
        (
            "getattr_dispatch",
            "import importlib\nvalue = getattr(importlib, 'import_module')('json')\n",
            "getattr(importlib, import_module)",
        ),
        (
            "importlib_dunder",
            "import importlib\nvalue = importlib.__import__('json')\n",
            "importlib.__import__",
        ),
        (
            "importlib_dunder_alias",
            "from importlib import __import__ as load\nvalue = load('json')\n",
            "load",
        ),
        (
            "builtin_assignment",
            "load = __import__\nvalue = load('json')\n",
            "load",
        ),
        (
            "importlib_star",
            "from importlib import *\nvalue = import_module('json')\n",
            "import_module",
        ),
        (
            "builtins_star",
            "from builtins import *\nvalue = __import__('json')\n",
            "__import__",
        ),
    ] {
        let fixture = BridgeFixture::new(case);
        fixture.write("dynamic.py", source);
        let diagnostics = discover_python_bridge_inventory(&fixture.package)
            .test_expect_err("dynamic import calls must fail inventory");
        assert!(diagnostics[0].message.contains(call));
        assert_eq!(
            diagnostics[0].code,
            DiagnosticCode::PYIMP_INVALID_BRIDGE_SOURCE
        );
    }
}

#[test]
fn dynamic_import_callable_reference_without_a_call_is_allowed() {
    let fixture = BridgeFixture::new("dynamic_reference");
    fixture.write(
        "reference.py",
        "import importlib\nresolver = importlib.import_module\nNAME = resolver.__name__\n",
    );

    discover_python_bridge_inventory(&fixture.package)
        .test_unwrap("a callable reference without a dynamic import call is valid");
}
