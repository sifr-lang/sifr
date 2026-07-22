import ast
import hashlib
import importlib
import importlib.metadata
import importlib.util
import inspect
import json
import pathlib
import sys
import sysconfig


def digest_bytes(value):
    return hashlib.sha256(value).hexdigest()


def identifier(value):
    return isinstance(value, str) and value.isidentifier()


def annotation_name(node):
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        parent = annotation_name(node.value)
        return None if parent is None else f"{parent}.{node.attr}"
    return None


def annotation_elements(node):
    if isinstance(node, ast.Tuple):
        return node.elts
    return [node]


def convert_annotation(node):
    if node is None:
        raise ValueError("missing type annotation")
    if isinstance(node, ast.Constant) and node.value is None:
        return "None"
    name = annotation_name(node)
    if name is not None:
        leaf = name.rsplit(".", 1)[-1]
        scalars = {
            "None": "None",
            "NoneType": "None",
            "bool": "bool",
            "int": "int",
            "float": "float",
            "str": "str",
            "bytes": "bytes",
        }
        if leaf in scalars:
            return scalars[leaf]
        if leaf in {"Any", "object"}:
            raise ValueError(f"untyped boundary {leaf!r} is forbidden")
        if leaf in {"Callable", "TypeVar", "Generic", "Protocol", "Self"}:
            raise ValueError(f"unsupported generic or callable annotation {name!r}")
        if identifier(leaf):
            return leaf
        raise ValueError(f"unsupported annotation {name!r}")
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.BitOr):
        return f"{convert_annotation(node.left)} | {convert_annotation(node.right)}"
    if not isinstance(node, ast.Subscript):
        raise ValueError(f"unsupported annotation syntax {ast.unparse(node)!r}")
    base = annotation_name(node.value)
    if base is None:
        raise ValueError(f"unsupported annotation syntax {ast.unparse(node)!r}")
    leaf = base.rsplit(".", 1)[-1]
    elements = annotation_elements(node.slice)
    if leaf == "Optional" and len(elements) == 1:
        return f"{convert_annotation(elements[0])} | None"
    if leaf == "Union" and len(elements) >= 2:
        return " | ".join(convert_annotation(element) for element in elements)
    if leaf in {"list", "List", "set", "Set"} and len(elements) == 1:
        container = leaf.lower()
        return f"{container}[{convert_annotation(elements[0])}]"
    if leaf in {"dict", "Dict"} and len(elements) == 2:
        return f"dict[{convert_annotation(elements[0])}, {convert_annotation(elements[1])}]"
    if leaf in {"tuple", "Tuple"} and elements:
        if any(isinstance(element, ast.Constant) and element.value is Ellipsis for element in elements):
            raise ValueError("variadic tuple annotations are not supported")
        return "tuple[" + ", ".join(convert_annotation(element) for element in elements) + "]"
    raise ValueError(f"unsupported generic annotation {base!r}")


def parameter_records(arguments):
    records = []
    positional = list(arguments.posonlyargs) + list(arguments.args)
    defaults = [None] * (len(positional) - len(arguments.defaults)) + list(arguments.defaults)
    positional_only_count = len(arguments.posonlyargs)
    for index, (argument, default) in enumerate(zip(positional, defaults)):
        records.append({
            "name": argument.arg,
            "type": convert_annotation(argument.annotation),
            "kind": "positional_only" if index < positional_only_count else "positional_or_keyword",
            "optional": default is not None,
        })
    if arguments.vararg is not None:
        records.append({
            "name": arguments.vararg.arg,
            "type": convert_annotation(arguments.vararg.annotation),
            "kind": "vararg",
            "optional": False,
        })
    for argument, default in zip(arguments.kwonlyargs, arguments.kw_defaults):
        records.append({
            "name": argument.arg,
            "type": convert_annotation(argument.annotation),
            "kind": "keyword_only",
            "optional": default is not None,
        })
    if arguments.kwarg is not None:
        records.append({
            "name": arguments.kwarg.arg,
            "type": convert_annotation(arguments.kwarg.annotation),
            "kind": "kwarg",
            "optional": False,
        })
    return records


def parsed_symbol(source, symbol):
    tree = ast.parse(source, type_comments=True)
    nodes = [
        node for node in module_declarations(tree.body)
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef))
        and node.name == symbol
    ]
    if not nodes:
        return None
    functions = [node for node in nodes if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))]
    overloads = [
        node for node in functions
        if any(annotation_name(decorator) == "overload" or annotation_name(decorator) == "typing.overload"
               for decorator in node.decorator_list)
    ]
    if overloads or len(functions) > 1:
        raise ValueError("overloaded symbols require an explicit single-signature override")
    node = nodes[-1]
    if isinstance(node, ast.ClassDef):
        if any(base for base in node.bases if annotation_name(base) in {"Generic", "typing.Generic"}):
            raise ValueError("generic classes are not supported")
        return {"name": symbol, "kind": "class", "async": False, "parameters": [], "return_type": None}
    return {
        "name": symbol,
        "kind": "function",
        "async": isinstance(node, ast.AsyncFunctionDef),
        "parameters": parameter_records(node.args),
        "return_type": convert_annotation(node.returns),
    }


def module_declarations(body):
    for node in body:
        if isinstance(node, ast.If):
            yield from module_declarations(node.body)
            yield from module_declarations(node.orelse)
        elif isinstance(node, ast.Try):
            yield from module_declarations(node.body)
            for handler in node.handlers:
                yield from module_declarations(handler.body)
            yield from module_declarations(node.orelse)
            yield from module_declarations(node.finalbody)
        else:
            yield node


def module_relative_candidates(module):
    relative = pathlib.Path(*module.split("."))
    return [relative.with_suffix(".pyi"), relative / "__init__.pyi"]


def stub_distribution_candidates(module):
    parts = module.split(".")
    top = parts[0]
    rest = pathlib.Path(*parts[1:]) if len(parts) > 1 else pathlib.Path()
    roots = [pathlib.Path(top), pathlib.Path(f"{top}-stubs")]
    candidates = []
    for root in roots:
        relative = root / rest
        candidates.extend([relative.with_suffix(".pyi"), relative / "__init__.pyi"])
    return candidates


def source_record(path, kind, identity):
    data = pathlib.Path(path).read_bytes()
    return {
        "path": str(path),
        "kind": kind,
        "identity": identity,
        "digest": digest_bytes(data),
        "text": data.decode("utf-8"),
    }


def source_candidates(config):
    module = config["module"]
    candidates = []
    for index, value in enumerate(config.get("overrides", [])):
        path = pathlib.Path(value)
        candidates.append(source_record(path, "override", f"override:{index}:{path.name}"))
    for distribution_name in config.get("stub_packages", []):
        distribution = importlib.metadata.distribution(distribution_name)
        wanted = {candidate.as_posix() for candidate in stub_distribution_candidates(module)}
        for entry in sorted(distribution.files or [], key=str):
            normalized = pathlib.PurePosixPath(str(entry)).as_posix()
            if normalized in wanted or any(normalized.endswith("/" + item) for item in wanted):
                path = distribution.locate_file(entry)
                candidates.append(source_record(
                    path,
                    "stub_package",
                    f"distribution:{distribution.metadata['Name']}=={distribution.version}:{normalized}",
                ))
    spec = importlib.util.find_spec(module)
    if spec is not None and spec.origin not in {None, "built-in", "frozen"}:
        origin = pathlib.Path(spec.origin)
        top_spec = importlib.util.find_spec(module.split(".")[0])
        package_root = None
        if top_spec is not None:
            if top_spec.submodule_search_locations:
                package_root = pathlib.Path(next(iter(top_spec.submodule_search_locations)))
            elif top_spec.origin:
                package_root = pathlib.Path(top_spec.origin).parent
        if package_root is not None and (package_root / "py.typed").is_file():
            inline = origin.with_suffix(".pyi") if origin.suffix != ".pyi" and origin.with_suffix(".pyi").is_file() else origin
            candidates.append(source_record(inline, "py_typed", f"py.typed:{module}:{inline.name}"))
    for index, root_value in enumerate(config.get("external_stubs", [])):
        root = pathlib.Path(root_value)
        paths = [root] if root.is_file() else [root / candidate for candidate in module_relative_candidates(module)]
        for path in paths:
            if path.is_file():
                candidates.append(source_record(path, "external_stub", f"external:{index}:{path.name}"))
                break
    return candidates


def runtime_distribution(module):
    top = module.split(".")[0]
    packages = importlib.metadata.packages_distributions()
    names = sorted(packages.get(top, []))
    if not names:
        return None
    distribution = importlib.metadata.distribution(names[0])
    return {"name": distribution.metadata["Name"], "version": distribution.version}


def introspected_symbol(module, symbol):
    target = getattr(importlib.import_module(module), symbol)
    if inspect.isclass(target):
        return {"name": symbol, "kind": "class", "async": False, "parameters": [], "return_type": None}
    if not callable(target):
        raise ValueError("selected target is not a callable or class")
    signature = inspect.signature(target)
    parameters = []
    kind_names = {
        inspect.Parameter.POSITIONAL_ONLY: "positional_only",
        inspect.Parameter.POSITIONAL_OR_KEYWORD: "positional_or_keyword",
        inspect.Parameter.VAR_POSITIONAL: "vararg",
        inspect.Parameter.KEYWORD_ONLY: "keyword_only",
        inspect.Parameter.VAR_KEYWORD: "kwarg",
    }
    for parameter in signature.parameters.values():
        if parameter.annotation is inspect.Parameter.empty:
            raise ValueError(f"parameter {parameter.name!r} has no runtime annotation")
        rendered = parameter.annotation if isinstance(parameter.annotation, str) else inspect.formatannotation(parameter.annotation)
        annotation = ast.parse(rendered, mode="eval").body
        parameters.append({
            "name": parameter.name,
            "type": convert_annotation(annotation),
            "kind": kind_names[parameter.kind],
            "optional": parameter.default is not inspect.Parameter.empty,
        })
    if signature.return_annotation is inspect.Signature.empty:
        raise ValueError("return value has no runtime annotation")
    return {
        "name": symbol,
        "kind": "function",
        "async": inspect.iscoroutinefunction(target),
        "parameters": parameters,
        "return_type": convert_annotation(ast.parse(
            signature.return_annotation if isinstance(signature.return_annotation, str)
            else inspect.formatannotation(signature.return_annotation),
            mode="eval",
        ).body),
    }


def main():
    config = json.loads(sys.argv[1])
    candidates = source_candidates(config)
    results = []
    errors = []
    for symbol in config["symbols"]:
        selected = None
        for candidate in candidates:
            try:
                declaration = parsed_symbol(candidate["text"], symbol)
            except (SyntaxError, UnicodeError, ValueError) as error:
                errors.append({"symbol": symbol, "reason": str(error), "source": candidate["identity"]})
                selected = "failed"
                break
            if declaration is not None:
                selected = {"declaration": declaration, "source": {key: candidate[key] for key in ("kind", "identity", "digest")}}
                break
        if selected is None:
            try:
                declaration = introspected_symbol(config["module"], symbol)
                encoded = json.dumps(declaration, sort_keys=True, separators=(",", ":")).encode()
                selected = {
                    "declaration": declaration,
                    "source": {
                        "kind": "introspection",
                        "identity": f"introspection:{config['module']}.{symbol}",
                        "digest": digest_bytes(encoded),
                    },
                }
            except (AttributeError, ImportError, TypeError, ValueError) as error:
                errors.append({"symbol": symbol, "reason": str(error), "source": "introspection"})
                selected = "failed"
        if selected != "failed":
            results.append(selected)
    report = {
        "module": config["module"],
        "soabi": sysconfig.get_config_var("SOABI") or "unknown",
        "distribution": runtime_distribution(config["module"]),
        "symbols": results,
        "errors": errors,
    }
    print(json.dumps(report, sort_keys=True, separators=(",", ":")))


if __name__ == "__main__":
    main()
