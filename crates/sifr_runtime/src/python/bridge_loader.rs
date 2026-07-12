use super::{PythonRuntimeError, PythonRuntimeError::ReservedBridgeCollision};
use pyo3::prelude::*;
use pyo3::types::PyDict;

const RUNTIME_ROOT: &str = "__sifr_bridge__";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonBridgeSource {
    pub module: String,
    pub source: String,
    pub filename: String,
    pub is_package: bool,
    pub package_prefix: String,
}

pub(super) fn install(
    py: Python<'_>,
    sources: &[PythonBridgeSource],
) -> Result<(), PythonRuntimeError> {
    reject_reserved_collisions(py)?;
    let entries = PyDict::new(py);
    for source in sources {
        entries
            .set_item(
                &source.module,
                (
                    &source.source,
                    &source.filename,
                    source.is_package,
                    &source.package_prefix,
                ),
            )
            .map_err(|error| loader_error(&error))?;
    }
    let module = PyModule::from_code(
        py,
        cr#"
import ast
import builtins
import importlib.util
import sys

_ROOT = "__sifr_bridge__"


class _BridgeImportRewriter(ast.NodeTransformer):
    def __init__(self, package_prefix):
        self.package_prefix = package_prefix

    def visit_Import(self, node):
        rewritten = []
        for alias in node.names:
            if alias.name == "bridge":
                rewritten.append(ast.Import(names=[ast.alias(
                    name=self.package_prefix,
                    asname=alias.asname or "bridge",
                )]))
            elif alias.name.startswith("bridge."):
                mapped = self.package_prefix + alias.name[len("bridge"):]
                if alias.asname is None:
                    rewritten.append(ast.Import(names=[ast.alias(
                        name=self.package_prefix,
                        asname="bridge",
                    )]))
                rewritten.append(ast.Import(names=[ast.alias(
                    name=mapped,
                    asname=alias.asname,
                )]))
            else:
                rewritten.append(ast.Import(names=[alias]))
        return [ast.copy_location(item, node) for item in rewritten]

    def visit_ImportFrom(self, node):
        if node.level == 0 and node.module is not None:
            if node.module == "bridge":
                node.module = self.package_prefix
            elif node.module.startswith("bridge."):
                node.module = self.package_prefix + node.module[len("bridge"):]
        return node


class _BridgeFinderLoader:
    _sifr_bridge_finder = True

    def __init__(self, entries):
        self.entries = entries

    def find_spec(self, fullname, path=None, target=None):
        if fullname == _ROOT or fullname.startswith(_ROOT + "."):
            entry = self.entries.get(fullname)
            is_package = bool(entry[2]) if entry is not None else False
            return importlib.util.spec_from_loader(fullname, self, is_package=is_package)
        return None

    def create_module(self, spec):
        return None

    def exec_module(self, module):
        entry = self.entries.get(module.__name__)
        if entry is None:
            raise ModuleNotFoundError(
                f"reserved Sifr bridge module {module.__name__!r} is not embedded",
                name=module.__name__,
            )
        source, filename, is_package, package_prefix = entry
        module.__file__ = filename
        if is_package:
            module.__path__ = []
        if source:
            tree = ast.parse(source, filename=filename, mode="exec")
            tree = _BridgeImportRewriter(package_prefix).visit(tree)
            ast.fix_missing_locations(tree)
            exec(compile(tree, filename, "exec"), module.__dict__)


def _put_finder_first(finder):
    sys.meta_path[:] = [item for item in sys.meta_path if item is not finder]
    sys.meta_path.insert(0, finder)


def install(entries):
    finder = _BridgeFinderLoader(dict(entries))
    _put_finder_first(finder)
    original_import = getattr(
        builtins,
        "__sifr_bridge_original_import__",
        builtins.__import__,
    )

    def guarded_import(name, globals=None, locals=None, fromlist=(), level=0):
        if name == _ROOT or name.startswith(_ROOT + "."):
            _put_finder_first(finder)
        return original_import(name, globals, locals, fromlist, level)

    builtins.__sifr_bridge_finder__ = finder
    builtins.__sifr_bridge_original_import__ = original_import
    builtins.__import__ = guarded_import
"#,
        c"<sifr-bridge-loader>",
        c"__sifr_bridge_loader__",
    )
    .map_err(|error| loader_error(&error))?;
    module
        .getattr("install")
        .and_then(|install| install.call1((entries,)))
        .map_err(|error| loader_error(&error))?;
    Ok(())
}

#[cfg(test)]
pub(super) fn reset_for_tests(py: Python<'_>) {
    let _ignored = py.run(
        cr#"
import builtins
import sys

original = getattr(builtins, "__sifr_bridge_original_import__", None)
if original is not None:
    builtins.__import__ = original
for name in ("__sifr_bridge_finder__", "__sifr_bridge_original_import__"):
    if hasattr(builtins, name):
        delattr(builtins, name)
sys.meta_path[:] = [
    item for item in sys.meta_path
    if not getattr(item, "_sifr_bridge_finder", False)
]
for name in list(sys.modules):
    if name == "__sifr_bridge__" or name.startswith("__sifr_bridge__."):
        del sys.modules[name]
"#,
        None,
        None,
    );
}

pub(super) fn ensure_first(py: Python<'_>) -> Result<(), PyErr> {
    let builtins = py.import("builtins")?;
    let Ok(finder) = builtins.getattr("__sifr_bridge_finder__") else {
        return Ok(());
    };
    let meta_path = py.import("sys")?.getattr("meta_path")?;
    if meta_path
        .call_method1("__contains__", (&finder,))?
        .extract::<bool>()?
    {
        meta_path.call_method1("remove", (&finder,))?;
    }
    meta_path.call_method1("insert", (0, &finder))?;
    Ok(())
}

fn reject_reserved_collisions(py: Python<'_>) -> Result<(), PythonRuntimeError> {
    let modules = py
        .import("sys")
        .and_then(|sys| sys.getattr("modules"))
        .and_then(|modules| modules.cast_into::<PyDict>().map_err(Into::into))
        .map_err(|error| loader_error(&error))?;
    for (name, _) in modules.iter() {
        let name = name
            .extract::<String>()
            .map_err(|error| loader_error(&error))?;
        if name == RUNTIME_ROOT || name.starts_with(&format!("{RUNTIME_ROOT}.")) {
            return Err(ReservedBridgeCollision { module: name });
        }
    }
    Ok(())
}

fn loader_error(error: &PyErr) -> PythonRuntimeError {
    PythonRuntimeError::PythonOperationFailed(format!(
        "install reserved Python bridge loader: {error}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::{
        initialize_cpython_with_config, initialize_runtime, reset_runtime_state_for_tests,
        test_config, test_guard,
    };

    const PACKAGE: &str = "__sifr_bridge__.p_test";

    #[test]
    fn loader_is_hermetic_rewrites_imports_and_restores_first_position() {
        let _guard = test_guard();
        reset_runtime_state_for_tests();
        let mut config = test_config("bridge-loader");
        config.bridge_sources = bridge_sources();
        initialize_cpython_with_config(&config).expect("CPython should initialize");

        Python::try_attach(|py| {
            let modules = py
                .import("sys")
                .and_then(|sys| sys.getattr("modules"))
                .and_then(|modules| modules.cast_into::<PyDict>().map_err(Into::into))
                .expect("sys.modules should be a dict");
            modules
                .set_item(RUNTIME_ROOT, py.None())
                .expect("collision fixture should insert");
            assert!(matches!(
                install(py, &config.bridge_sources),
                Err(ReservedBridgeCollision { .. })
            ));
            modules
                .del_item(RUNTIME_ROOT)
                .expect("collision fixture should clear");
        })
        .expect("CPython should be attached");

        initialize_runtime(config).expect("runtime should initialize with embedded bridges");
        Python::try_attach(|py| {
            let module = py
                .import(&format!("{PACKAGE}.main"))
                .expect("embedded main should import");
            let value = module
                .getattr("compute")
                .and_then(|function| function.call0())
                .and_then(|value| value.extract::<i32>())
                .expect("rewritten sibling import should execute");
            assert_eq!(value, 42);
            let filename = module
                .getattr("compute")
                .and_then(|function| function.getattr("__code__"))
                .and_then(|code| code.getattr("co_filename"))
                .and_then(|filename| filename.extract::<String>())
                .expect("compiled function should expose a filename");
            assert_eq!(filename, format!("<{PACKAGE}.main>"));

            py.import("sys")
                .and_then(|sys| sys.getattr("meta_path"))
                .and_then(|meta_path| meta_path.call_method1("pop", (0,)))
                .expect("test should mutate sys.meta_path");
        })
        .expect("CPython should be attached");

        let target = crate::python::resolve_target(&[
            "__sifr_bridge__".to_string(),
            "p_test".to_string(),
            "late".to_string(),
            "answer".to_string(),
        ])
        .expect("reserved resolution should restore the finder");
        let value = crate::python::call_object(&target, &[], &[])
            .and_then(|value| crate::python::to_i32(&value))
            .expect("late embedded target should execute");
        assert_eq!(value, 7);
    }

    fn bridge_sources() -> Vec<PythonBridgeSource> {
        [
            (RUNTIME_ROOT, "", true, RUNTIME_ROOT),
            (PACKAGE, "", true, PACKAGE),
            (&format!("{PACKAGE}.helper"), "VALUE = 41\n", false, PACKAGE),
            (
                &format!("{PACKAGE}.main"),
                "import bridge.helper\n\ndef compute():\n    return bridge.helper.VALUE + 1\n",
                false,
                PACKAGE,
            ),
            (
                &format!("{PACKAGE}.late"),
                "def answer():\n    return 7\n",
                false,
                PACKAGE,
            ),
        ]
        .into_iter()
        .map(
            |(module, source, is_package, package_prefix)| PythonBridgeSource {
                module: module.to_string(),
                source: source.to_string(),
                filename: format!("<{module}>"),
                is_package,
                package_prefix: package_prefix.to_string(),
            },
        )
        .collect()
    }
}
