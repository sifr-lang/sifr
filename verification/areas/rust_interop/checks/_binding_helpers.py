"""Shared Sifr binding-source helpers for Rust interop fixture checks."""

from __future__ import annotations

import re

FIXTURE_BINDING_TOKENS = {
    "proc_macro_trust": "bridge.generated",
    "ecosystem_backend_certification": "bridge.backend",
    "ecosystem_cli_certification": "bridge.cli",
}


def package_example_binding_token(fixture_id: str, crate_token: str) -> str:
    return FIXTURE_BINDING_TOKENS.get(fixture_id, crate_token)


def rust_bound_declarations(text: str) -> list[tuple[str, str]]:
    names: list[str] = []
    declarations: list[tuple[str, str]] = []
    decorators: list[str] = []
    for line in text.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("@"):
            decorators.append(stripped)
            continue
        if is_rust_decorated_binding(stripped, decorators):
            name = decorated_function_name(stripped)
            if name is not None and name not in names:
                names.append(name)
                declarations.append((name, decorated_function_return_type(stripped)))
        if stripped and not stripped.startswith("@"):
            decorators = []
    return declarations


def rust_bound_declaration_names(text: str) -> list[str]:
    return [name for name, _return_type in rust_bound_declarations(text)]


def is_rust_decorated_binding(stripped: str, decorators: list[str]) -> bool:
    return (
        any(decorator.startswith("@rust") for decorator in decorators)
        and (stripped.startswith("def ") or stripped.startswith("async def "))
        and stripped.endswith(": ...")
    )


def decorated_function_name(stripped: str) -> str | None:
    if stripped.startswith("async def "):
        stripped = stripped.removeprefix("async ")
    if not stripped.startswith("def "):
        return None
    declaration_head = stripped.removeprefix("def ").split("(", maxsplit=1)[0].strip()
    return declaration_head.split("[", maxsplit=1)[0].strip()


def decorated_function_return_type(stripped: str) -> str:
    if "->" not in stripped:
        return "None"
    return stripped.split("->", maxsplit=1)[1].rsplit(":", maxsplit=1)[0].strip()


def verifier_binds_call(verifier_body: str, bound_function: str) -> bool:
    for line in verifier_body.splitlines():
        for before_call in bound_call_prefixes(line, bound_function):
            if is_assignment_prefix(before_call) and not before_call.lstrip().startswith("return "):
                return True
    return False


def is_assignment_prefix(prefix: str) -> bool:
    return re.search(r"(?<![=!<>])=(?![=])", prefix) is not None


def bound_call_prefixes(line: str, bound_function: str) -> list[str]:
    prefixes: list[str] = []
    marker = f"{bound_function}("
    start = 0
    while True:
        index = line.find(marker, start)
        if index < 0:
            break
        if index == 0 or not is_identifier_or_path_char(line[index - 1]):
            prefixes.append(line[:index])
        start = index + len(marker)
    method_marker = f".{bound_function}("
    start = 0
    while True:
        index = line.find(method_marker, start)
        if index < 0:
            break
        prefixes.append(line[: index + 1])
        start = index + len(method_marker)
    return prefixes


def is_identifier_or_path_char(char: str) -> bool:
    return char.isalnum() or char in {"_", "."}


def contains_empty_pass_body(text: str) -> bool:
    return any(line.strip() == "pass" for line in text.splitlines())
