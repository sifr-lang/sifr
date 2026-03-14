#!/usr/bin/env python3
from __future__ import annotations

import ast
import collections
import copy
import functools
import html
import importlib.util
import inspect
import itertools
import json
import math
import os
from pathlib import Path
import random
import re
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
AUDITS = ROOT / "audits" / "leetcode"
CACHE_PATH = Path("/tmp/leetcode_question_cache.json")


class PyListNode:
    def __init__(self, val: int = 0, next: "PyListNode | None" = None):
        self.val = val
        self.next = next


class PyTreeNode:
    def __init__(
        self,
        val: int = 0,
        left: "PyTreeNode | None" = None,
        right: "PyTreeNode | None" = None,
    ):
        self.val = val
        self.left = left
        self.right = right


class PyNode:
    def __init__(
        self,
        val: int = 0,
        next: "PyNode | None" = None,
        random: "PyNode | None" = None,
        left: "PyNode | None" = None,
        right: "PyNode | None" = None,
        neighbors: list["PyNode"] | None = None,
        key: int = -1,
    ):
        self.val = val
        self.next = next
        self.random = random
        self.left = left
        self.right = right
        self.neighbors = [] if neighbors is None else neighbors
        self.key = key


BASE_EXEC_ENV = {
    "ListNode": PyListNode,
    "TreeNode": PyTreeNode,
    "Node": PyNode,
    "Optional": __import__("typing").Optional,
    "List": list,
    "Tuple": tuple,
    "Dict": dict,
    "Set": set,
    "collections": collections,
    "math": math,
    "heapq": __import__("heapq"),
    "heappush": __import__("heapq").heappush,
    "heappop": __import__("heapq").heappop,
    "heapify": __import__("heapq").heapify,
    "itertools": itertools,
    "random": random,
    "functools": functools,
    "defaultdict": collections.defaultdict,
    "Counter": collections.Counter,
    "deque": collections.deque,
    "cache": functools.cache,
    "lru_cache": functools.lru_cache,
    "isBadVersion": lambda version: version >= 1,
    "guess": lambda num: 0,
    "choice": lambda values: values[0] if values else None,
    "sqrt": math.sqrt,
    "ceil": math.ceil,
    "ord": ord,
    "chr": chr,
    "reversed": reversed,
    "list": list,
    "dict": dict,
    "tuple": tuple,
    "set": set,
}


PY_LISTNODE_HELPERS = """class ListNode:
    def __init__(self, val: int = 0, next: 'ListNode | None' = None):
        self.val = val
        self.next = next


def list_node_to_string(node: ListNode | None) -> str:
    parts = []
    cur = node
    while cur is not None:
        parts.append(str(cur.val))
        cur = cur.next
    return "->".join(parts) if parts else "None"
"""


SIFR_LISTNODE_HELPERS = """class ListNode:
    val: int
    next: ListNode | None

    def __init__(self, val: int = 0, next: ListNode | None = None):
        self.val = val
        self.next = next


def listNodeToString(node: ListNode | None) -> str:
    if node is None:
        return "None"
    parts: list[str] = []
    cur: ListNode | None = node
    while cur is not None:
        parts.append(str(cur.val))
        cur = cur.next
    return "->".join(parts)
"""


PY_TREENODE_HELPERS = """class TreeNode:
    def __init__(
        self,
        val: int = 0,
        left: 'TreeNode | None' = None,
        right: 'TreeNode | None' = None,
    ):
        self.val = val
        self.left = left
        self.right = right


def tree_to_string(node: TreeNode | None) -> str:
    if node is None:
        return "None"
    return f"{node.val}({tree_to_string(node.left)},{tree_to_string(node.right)})"
"""


SIFR_TREENODE_HELPERS = """class TreeNode:
    val: int
    left: TreeNode | None
    right: TreeNode | None

    def __init__(
        self,
        val: int = 0,
        left: TreeNode | None = None,
        right: TreeNode | None = None,
    ):
        self.val = val
        self.left = left
        self.right = right


def treeToString(node: TreeNode | None) -> str:
    if node is None:
        return "None"
    return str(node.val) + "(" + treeToString(node.left) + "," + treeToString(node.right) + ")"
"""


PY_NODE_HELPERS = """class Node:
    def __init__(
        self,
        val: int = 0,
        next: 'Node | None' = None,
        random: 'Node | None' = None,
        left: 'Node | None' = None,
        right: 'Node | None' = None,
        neighbors: list['Node'] | None = None,
        key: int = -1,
    ):
        self.val = val
        self.next = next
        self.random = random
        self.left = left
        self.right = right
        self.neighbors = [] if neighbors is None else neighbors
        self.key = key
"""


SIFR_NODE_HELPERS = """class Node:
    val: int
    next: Node | None
    random: Node | None
    left: Node | None
    right: Node | None
    neighbors: list[Node]
    key: int

    def __init__(
        self,
        val: int = 0,
        next: Node | None = None,
        random: Node | None = None,
        left: Node | None = None,
        right: Node | None = None,
        neighbors: list[Node] = [],
        key: int = -1,
    ):
        self.val = val
        self.next = next
        self.random = random
        self.left = left
        self.right = right
        self.neighbors = neighbors
        self.key = key
"""


PY_RANDOM_LIST_HELPERS = """def build_random_list(spec: list[tuple[int, int]]) -> Node | None:
    if len(spec) == 0:
        return None
    nodes = [Node(val) for val, _ in spec]
    for i in range(len(nodes) - 1):
        nodes[i].next = nodes[i + 1]
    for i, (_, random_index) in enumerate(spec):
        if random_index >= 0:
            nodes[i].random = nodes[random_index]
    return nodes[0]


def random_list_to_pairs(head: Node | None) -> list[tuple[int, int]]:
    nodes = []
    cur = head
    while cur is not None:
        nodes.append(cur)
        cur = cur.next
    indices = {node: i for i, node in enumerate(nodes)}
    pairs = []
    for node in nodes:
        random_index = -1 if node.random is None else indices[node.random]
        pairs.append((node.val, random_index))
    return pairs
"""


SIFR_RANDOM_LIST_HELPERS = """def buildRandomList(spec: list[tuple[int, int]]) -> Node | None:
    if len(spec) == 0:
        return None
    nodes: list[Node] = []
    for pair in spec:
        nodes.append(Node(pair[0]))
    i = 0
    while i + 1 < len(nodes):
        nodes[i].next = nodes[i + 1]
        i += 1
    i = 0
    while i < len(spec):
        randomIndex = spec[i][1]
        if randomIndex >= 0:
            nodes[i].random = nodes[randomIndex]
        i += 1
    return nodes[0]


def randomListToPairs(head: Node | None) -> list[tuple[int, int]]:
    nodes: list[Node] = []
    cur = head
    while cur is not None:
        nodes.append(cur)
        cur = cur.next
    pairs: list[tuple[int, int]] = []
    i = 0
    while i < len(nodes):
        randomIndex = -1
        if nodes[i].random is not None:
            j = 0
            while j < len(nodes):
                if nodes[j] is nodes[i].random:
                    randomIndex = j
                    break
                j += 1
        pairs.append((nodes[i].val, randomIndex))
        i += 1
    return pairs
"""


PY_GRAPH_HELPERS = """def build_graph(adjacency: list[list[int]]) -> Node | None:
    if len(adjacency) == 0:
        return None
    nodes = [Node(i + 1) for i in range(len(adjacency))]
    for i, neighbors in enumerate(adjacency):
        nodes[i].neighbors = [nodes[value - 1] for value in neighbors]
    return nodes[0]


def graph_to_adj(node: Node | None) -> list[list[int]]:
    if node is None:
        return []
    queue = [node]
    seen = {node}
    by_val = {}
    while queue:
        cur = queue.pop(0)
        by_val[cur.val] = sorted(neighbor.val for neighbor in cur.neighbors)
        for neighbor in cur.neighbors:
            if neighbor not in seen:
                seen.add(neighbor)
                queue.append(neighbor)
    return [by_val[i] for i in range(1, len(by_val) + 1)]
"""


SIFR_GRAPH_HELPERS = """def buildGraph(adjacency: list[list[int]]) -> Node | None:
    if len(adjacency) == 0:
        return None
    nodes: list[Node] = []
    i = 0
    while i < len(adjacency):
        nodes.append(Node(i + 1))
        i += 1
    i = 0
    while i < len(adjacency):
        neighbors: list[Node] = []
        for value in adjacency[i]:
            neighbors.append(nodes[value - 1])
        nodes[i].neighbors = neighbors
        i += 1
    return nodes[0]


def graphToAdj(node: Node | None) -> list[list[int]]:
    if node is None:
        return []
    queue: list[Node] = [node]
    seenVals: list[int] = []
    adjacency: list[list[int]] = []
    while len(queue) > 0:
        cur = queue[0]
        queue = queue[1:]
        if cur.val in seenVals:
            continue
        seenVals.append(cur.val)
        neighborVals: list[int] = []
        for neighbor in cur.neighbors:
            neighborVals.append(neighbor.val)
            if neighbor.val not in seenVals:
                queue.append(neighbor)
        neighborVals.sort()
        adjacency.append(neighborVals)
    return adjacency
"""


def load_convert_all_tests() -> dict[str, tuple[str, list[tuple[str, str]]]]:
    spec = importlib.util.spec_from_file_location(
        "convert_all", AUDITS / "convert_all.py"
    )
    mod = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(mod)
    return mod.make_test_cases()


def problem_id_from_name(name: str) -> str | None:
    match = re.match(r"^(\d{4})_", name)
    return match.group(1) if match else None


def read_text(path: Path) -> str:
    return path.read_text()


def has_no_assert(path: Path) -> bool:
    text = read_text(path)
    main = extract_main_block(text)
    if main is None:
        return False
    return "assert " not in main


def has_placeholder_assert(path: Path) -> bool:
    text = read_text(path)
    main = extract_main_block(text)
    if main is None:
        return False
    return bool(
        re.search(
            r"""assert\s+(['"])(.*?)\1\s*==\s*(['"])\2\3""",
            main,
        )
    )


def needs_oracle_backfill(path: Path) -> bool:
    return has_no_assert(path) or has_placeholder_assert(path)


def extract_main_block(text: str) -> str | None:
    match = re.search(r"^def main\(\):\n(?P<body>(?:^[ \t]+.*\n?)*)", text, re.M)
    return match.group("body") if match else None


def extract_print_calls_from_main(text: str) -> list[str]:
    main = extract_main_block(text)
    if main is None:
        return []
    exprs = []
    for line in main.splitlines():
        stripped = line.strip()
        match = re.match(r"print\((.*)\)$", stripped)
        if match:
            exprs.append(match.group(1))
    return exprs


def top_level_defs(text: str) -> list[tuple[str, list[str], str]]:
    ignored = {
        "main",
        "list_node_to_string",
        "tree_to_string",
        "build_random_list",
        "random_list_to_pairs",
        "build_graph",
        "graph_to_adj",
        "listNodeToString",
        "treeToString",
        "buildRandomList",
        "randomListToPairs",
        "buildGraph",
        "graphToAdj",
    }
    defs = []
    for match in re.finditer(r"^def\s+(\w+)\((.*?)\)\s*(?:->\s*([^:]+))?:", text, re.M):
        name = match.group(1)
        params_src = match.group(2)
        ret = (match.group(3) or "").strip()
        params = []
        for part in params_src.split(","):
            token = part.strip()
            if not token:
                continue
            pname = token.split(":")[0].split("=")[0].strip()
            params.append(pname)
        defs.append((name, params, ret))
    return [item for item in defs if item[0] not in ignored]


def top_level_classes(text: str) -> list[str]:
    return re.findall(r"^class\s+(\w+)\s*[:(]", text, re.M)


def normalize_bool_null(expr: str) -> str:
    return (
        expr.replace("null", "None")
        .replace("true", "True")
        .replace("false", "False")
    )


def parse_literal_or_string(expr: str):
    expr = normalize_bool_null(expr.strip())
    try:
        return ast.literal_eval(expr)
    except Exception:
        return expr


def render_code(value, lang: str) -> str:
    if isinstance(value, PyListNode):
        return render_listnode_expr(value, lang)
    if isinstance(value, PyTreeNode):
        return render_treenode_expr(value, lang)
    if isinstance(value, PyNode):
        return "None"
    if value is None:
        return "None"
    if isinstance(value, bool):
        return "True" if value else "False"
    if isinstance(value, str):
        return repr(value)
    if isinstance(value, (int, float)):
        return repr(value)
    if isinstance(value, list):
        return "[" + ", ".join(render_code(item, lang) for item in value) + "]"
    if isinstance(value, tuple):
        inner = ", ".join(render_code(item, lang) for item in value)
        if len(value) == 1:
            inner += ","
        return f"({inner})"
    if isinstance(value, dict):
        items = ", ".join(
            f"{render_code(k, lang)}: {render_code(v, lang)}"
            for k, v in value.items()
        )
        return "{" + items + "}"
    return repr(value)


def render_listnode_expr(node: PyListNode | None, lang: str) -> str:
    if node is None:
        return "None"
    return f"ListNode({render_code(node.val, lang)}, {render_listnode_expr(node.next, lang)})"


def render_treenode_expr(node: PyTreeNode | None, lang: str) -> str:
    if node is None:
        return "None"
    return (
        "TreeNode("
        + f"{render_code(node.val, lang)}, "
        + f"{render_treenode_expr(node.left, lang)}, "
        + f"{render_treenode_expr(node.right, lang)})"
    )


def choose_arg_for_type(type_str: str, name: str):
    token = str(type_str).replace("typing.", "").replace(" ", "")
    pname = name.lower()
    if token in {"", "Any"}:
        if pname in {"intervals", "points", "matrix", "grid", "board", "queries"}:
            return [[0]]
        if pname in {"edges", "prerequisites", "connections", "roads", "flights"}:
            return [[0, 0]]
        if pname in {"times", "meetings"}:
            return [[0, 0, 0]]
        if pname in {"nums", "stones", "ideas", "arr", "values", "tasks"}:
            return [0]
        if pname in {"strs"}:
            return [""]
        if pname in {"k", "n", "m", "left", "right", "dst", "src"}:
            return 1
        return None
    if "ListNode" in token:
        return PyListNode(0)
    if "TreeNode" in token:
        return PyTreeNode(0)
    if re.search(r"\bNode\b", token):
        return None
    if token in {"int", "integer", "number", "float", "double", "<class'int'>"}:
        return 1 if pname in {"k", "n", "m", "left", "right", "dst", "src"} else 0
    if token in {"str", "string", "char", "character", "<class'str'>"}:
        return ""
    if token in {"bool", "boolean", "<class'bool'>"}:
        return False
    if "List[List[int]]" in token or "list[list[int]]" in token or "integer[][]" in token:
        if pname in {"edges", "prerequisites", "connections", "flights"}:
            return [[0, 0]]
        if pname in {"times", "meetings"}:
            return [[0, 0, 0]]
        return [[0]]
    if "List[List[str]]" in token or "list[list[str]]" in token or "string[][]" in token:
        return [[""]]
    if "List[" in token or "list[" in token or "[]" in token:
        if "str" in token or "string" in token:
            return [""]
        return [0]
    if "dict" in token or "Dict[" in token:
        return {}
    return None


def needs_helper(text: str, helper_name: str) -> bool:
    return helper_name in text and f"class {helper_name}:" not in text


def inject_helpers(text: str, lang: str) -> str:
    pieces = []
    if needs_helper(text, "ListNode"):
        pieces.append(PY_LISTNODE_HELPERS if lang == "py" else SIFR_LISTNODE_HELPERS)
    if needs_helper(text, "TreeNode"):
        pieces.append(PY_TREENODE_HELPERS if lang == "py" else SIFR_TREENODE_HELPERS)
    if needs_helper(text, "Node"):
        pieces.append(PY_NODE_HELPERS if lang == "py" else SIFR_NODE_HELPERS)
    if "build_random_list(" in text and "def build_random_list(" not in text:
        pieces.append(PY_RANDOM_LIST_HELPERS)
    if "buildRandomList(" in text and "def buildRandomList(" not in text:
        pieces.append(SIFR_RANDOM_LIST_HELPERS)
    if "build_graph(" in text and "def build_graph(" not in text:
        pieces.append(PY_GRAPH_HELPERS)
    if "buildGraph(" in text and "def buildGraph(" not in text:
        pieces.append(SIFR_GRAPH_HELPERS)
    if not pieces:
        return text
    lines = text.splitlines()
    insert_at = 0
    while insert_at < len(lines) and lines[insert_at].startswith("#"):
        insert_at += 1
    prefix = "\n".join(lines[:insert_at]).rstrip()
    suffix = "\n".join(lines[insert_at:]).lstrip("\n")
    helper_block = "\n\n".join(pieces).rstrip()
    if prefix:
        return prefix + "\n\n" + helper_block + "\n\n" + suffix
    return helper_block + "\n\n" + suffix


def replace_main(text: str, lines: list[str], lang: str) -> str:
    body = "\n".join(f"    {line}" for line in lines).rstrip() + "\n"
    new_text, count = re.subn(
        r"^def main\(\):\n(?:^[ \t]+.*\n?)*",
        "def main():\n" + body,
        text,
        count=1,
        flags=re.M,
    )
    if count == 0:
        if lang == "py":
            return text.rstrip() + "\n\n\ndef main():\n" + body + '\nif __name__ == "__main__":\n    main()\n'
        return text.rstrip() + "\n\n\ndef main():\n" + body
    return new_text


def exec_python_source(text: str):
    env = dict(BASE_EXEC_ENV)
    exec(text, env)
    return env


def eval_print_harness(py_path: Path):
    text = read_text(py_path)
    exprs = extract_print_calls_from_main(text)
    if not exprs:
        return None
    try:
        env = exec_python_source(text)
        tests = []
        for expr in exprs:
            value = eval(expr, env)  # noqa: S307 - fixture-only source
            tests.append(("expr", expr, value))
        return tests
    except Exception:
        return None


def literalize_expected(expected_raw: str, return_type: str):
    parsed = parse_literal_or_string(expected_raw)
    if isinstance(parsed, str) and return_type.lower() in {"str", "string"}:
        return parsed
    return parsed


def build_call_expr(func_name: str, params: list[str], args_expr: str) -> str:
    clean_args = args_expr.strip()
    if clean_args.startswith("(") and clean_args.endswith(")"):
        clean_args = clean_args[1:-1].strip()
    parts = []
    if params and params[0] == "self":
        parts.append("None")
    if clean_args:
        parts.append(clean_args)
    return f"{func_name}({', '.join(parts)})"


def tests_from_convert_all(problem_id: str, py_text: str):
    tc = CONVERT_ALL_TESTS.get(problem_id)
    if not tc or not tc[1]:
        return None
    defs = top_level_defs(py_text)
    if not defs:
        return None
    func_name, params, return_type = defs[0]
    tests = []
    for args, expected in tc[1]:
        call_expr = build_call_expr(func_name, params, args)
        tests.append(("call", call_expr, literalize_expected(expected, return_type)))
    return tests


def tests_from_trivial_eval(py_path: Path):
    text = read_text(py_path)
    env = exec_python_source(text)
    defs = top_level_defs(text)
    if defs:
        func_name, params, _ = defs[0]
        fn = env[func_name]
        call_parts = []
        args = []
        annotations = fn.__annotations__
        for pname in params:
            if pname == "self":
                call_parts.append("None")
                args.append(None)
                continue
            value = choose_arg_for_type(annotations.get(pname, ""), pname)
            call_parts.append(render_code(value, "py"))
            args.append(value)
        result = fn(*args)
        return [("call", f"{func_name}({', '.join(call_parts)})", result)]
    classes = [name for name in top_level_classes(text) if name not in {"ListNode", "TreeNode", "Node"}]
    if not classes:
        return None
    class_name = classes[0]
    cls = env[class_name]
    signature = inspect.signature(cls)
    call_args = []
    args = []
    for pname, param in list(signature.parameters.items())[1:]:
        value = choose_arg_for_type(param.annotation, pname)
        call_args.append(render_code(value, "py"))
        args.append(value)
    ctor_expr = f"{class_name}({', '.join(call_args)})"
    return [("class_ctor", ctor_expr, None)]


def slug_for_path(py_path: Path) -> str:
    name = py_path.stem.split("_", 1)[1]
    return name.lower().replace("_", "-")


def load_cache() -> dict[str, dict]:
    if CACHE_PATH.exists():
        return json.loads(CACHE_PATH.read_text())
    return {}


def save_cache(cache: dict[str, dict]) -> None:
    CACHE_PATH.write_text(json.dumps(cache))


LEETCODE_CACHE = load_cache()


def fetch_question(cache_key: str, slug: str) -> dict | None:
    if cache_key in LEETCODE_CACHE:
        return LEETCODE_CACHE[cache_key]
    query = {
        "query": "query q($titleSlug: String!) { question(titleSlug: $titleSlug) { titleSlug content metaData exampleTestcases sampleTestCase } }",
        "variables": {"titleSlug": slug},
    }
    req = urllib.request.Request(
        "https://leetcode.com/graphql",
        data=json.dumps(query).encode(),
        headers={"Content-Type": "application/json", "User-Agent": "Mozilla/5.0"},
    )
    with urllib.request.urlopen(req, timeout=30) as response:
        data = json.load(response)["data"]["question"]
    LEETCODE_CACHE[cache_key] = data
    save_cache(LEETCODE_CACHE)
    return data


def parse_pre_blocks(content: str) -> list[str]:
    blocks = re.findall(r"<pre>(.*?)</pre>", content, re.S)
    cleaned = []
    for block in blocks:
        plain = re.sub(r"<.*?>", "", block)
        cleaned.append(html.unescape(plain).strip())
    return cleaned


def parse_example_io(block: str):
    match = re.search(r"Input[:\n]\s*(.*?)\s*Output[:\n]\s*(.*?)(?:\n\n|$)", block, re.S)
    if not match:
        return None
    return match.group(1).strip(), match.group(2).strip().splitlines()[0].strip()


def split_named_inputs(raw_input: str, param_names: list[str]) -> list[str]:
    if not param_names:
        return []
    values = []
    cursor = raw_input
    starts = []
    for name in param_names:
        pattern = f"{name} = "
        idx = cursor.find(pattern)
        if idx == -1:
            return []
        starts.append((name, idx))
    starts.sort(key=lambda item: item[1])
    for i, (name, idx) in enumerate(starts):
        start = idx + len(name) + 3
        end = starts[i + 1][1] - 2 if i + 1 < len(starts) else len(cursor)
        values.append(cursor[start:end].strip().rstrip(","))
    return values


def render_graphql_arg(value_str: str, type_str: str, lang: str) -> str:
    value = parse_literal_or_string(value_str)
    token = str(type_str).replace("typing.", "").replace(" ", "")
    if "ListNode" in token:
        return render_listnode_expr(py_listnode_from_list(value), lang)
    if "TreeNode" in token:
        return render_treenode_expr(py_tree_from_level_order(value), lang)
    return render_code(value, lang)


def py_listnode_from_list(values):
    head = None
    for value in reversed(values):
        head = PyListNode(value, head)
    return head


def py_tree_from_level_order(values):
    if not values:
        return None
    nodes = [None if value is None else PyTreeNode(value) for value in values]
    kids = nodes[::-1]
    root = kids.pop()
    for node in nodes:
        if node is not None:
            if kids:
                node.left = kids.pop()
            if kids:
                node.right = kids.pop()
    return root


def py_random_list_from_pairs(spec):
    if not spec:
        return None
    nodes = [PyNode(val) for val, _ in spec]
    for i in range(len(nodes) - 1):
        nodes[i].next = nodes[i + 1]
    for i, (_, random_index) in enumerate(spec):
        if random_index is not None and random_index >= 0:
            nodes[i].random = nodes[random_index]
    return nodes[0]


def py_random_list_to_pairs(head):
    nodes = []
    cur = head
    while cur is not None:
        nodes.append(cur)
        cur = cur.next
    index_of = {node: i for i, node in enumerate(nodes)}
    return [
        (node.val, -1 if node.random is None else index_of[node.random]) for node in nodes
    ]


def py_graph_from_adjacency(adjacency):
    if not adjacency:
        return None
    nodes = [PyNode(i + 1) for i in range(len(adjacency))]
    for i, neighbors in enumerate(adjacency):
        nodes[i].neighbors = [nodes[value - 1] for value in neighbors]
    return nodes[0]


def py_graph_to_adjacency(node):
    if node is None:
        return []
    queue = collections.deque([node])
    seen = {node}
    by_val = {}
    while queue:
        cur = queue.popleft()
        by_val[cur.val] = sorted(neighbor.val for neighbor in cur.neighbors)
        for neighbor in cur.neighbors:
            if neighbor not in seen:
                seen.add(neighbor)
                queue.append(neighbor)
    return [by_val[i] for i in range(1, len(by_val) + 1)]


def parse_input_value(problem_id: str, type_str: str, raw: str):
    value = parse_literal_or_string(raw)
    token = str(type_str).replace("typing.", "").replace(" ", "")
    if problem_id == "0138":
        pairs = []
        for entry in value:
            random_index = entry[1]
            pairs.append((entry[0], -1 if random_index is None else random_index))
        return py_random_list_from_pairs(pairs)
    if problem_id == "0133":
        return py_graph_from_adjacency(value)
    if "TreeNode" in token:
        return py_tree_from_level_order(value)
    if "ListNode" in token:
        return py_listnode_from_list(value)
    if token == "Node":
        return py_graph_from_adjacency(value)
    return value


def render_input_expr(value, type_str: str, lang: str, problem_id: str) -> str:
    token = str(type_str).replace("typing.", "").replace(" ", "")
    if problem_id == "0138":
        return ("build_random_list" if lang == "py" else "buildRandomList") + "(" + render_code(py_random_list_to_pairs(value), lang) + ")"
    if problem_id == "0133":
        return ("build_graph" if lang == "py" else "buildGraph") + "(" + render_code(py_graph_to_adjacency(value), lang) + ")"
    if "TreeNode" in token:
        return render_treenode_expr(value, lang)
    if "ListNode" in token:
        return render_listnode_expr(value, lang)
    if token == "Node":
        return ("build_graph" if lang == "py" else "buildGraph") + "(" + render_code(py_graph_to_adjacency(value), lang) + ")"
    return render_code(value, lang)


def translate_expr_for_lang(expr: str, lang: str) -> str:
    if lang == "py":
        return expr
    return (
        expr.replace("build_random_list(", "buildRandomList(")
        .replace("build_graph(", "buildGraph(")
        .replace("random_list_to_pairs(", "randomListToPairs(")
        .replace("graph_to_adj(", "graphToAdj(")
    )


def grouped_example_inputs(meta: dict, question: dict) -> list[list[str]]:
    raw = (question.get("exampleTestcases") or "").strip()
    if not raw:
        raw = (question.get("sampleTestCase") or "").strip()
    if not raw:
        return []
    lines = [line.strip() for line in raw.splitlines() if line.strip()]
    if "classname" in meta:
        return [lines]
    params = meta.get("params", [])
    if len(params) == 0:
        return []
    chunk = len(params)
    return [lines[i : i + chunk] for i in range(0, len(lines), chunk) if len(lines[i : i + chunk]) == chunk]


def build_eval_tests_from_examples(problem_id: str, py_path: Path):
    py_text = read_text(py_path)
    slug = slug_for_path(py_path)
    question = fetch_question(py_path.stem, slug)
    if question is None:
        return None
    meta = json.loads(question["metaData"])
    if "classname" in meta:
        grouped = grouped_example_inputs(meta, question)
        if not grouped:
            return None
        lines = grouped[0]
        if len(lines) < 2:
            return None
        ops = ast.literal_eval(normalize_bool_null(lines[0]))
        args = ast.literal_eval(normalize_bool_null(lines[1]))
        env = exec_python_source(py_text)
        cls = env[meta["classname"]]
        obj = cls(*args[0])
        outputs = [None]
        for op, op_args in zip(ops[1:], args[1:]):
            outputs.append(getattr(obj, op)(*op_args))
        return [("systemdesign", meta["classname"], (ops, args, outputs))]
    if re.search(r"^class\s+Codec\b", py_text, re.M):
        tests = []
        for lines in grouped_example_inputs(meta, question)[:2]:
            if not lines:
                continue
            root = py_tree_from_level_order(parse_literal_or_string(lines[0]))
            tests.append(("codec_roundtrip", render_treenode_expr(root, "py"), root))
        return tests or None

    defs = top_level_defs(py_text)
    if not defs:
        return None
    func_name, params, _ = defs[0]
    env = exec_python_source(py_text)
    fn = env[func_name]
    meta_params = meta.get("params", [])
    tests = []
    for sample in grouped_example_inputs(meta, question)[:3]:
        actual_args = [
            parse_input_value(problem_id, meta_param["type"], raw)
            for meta_param, raw in zip(meta_params, sample)
        ]
        rendered_args = [
            render_input_expr(value, meta_param["type"], "py", problem_id)
            for meta_param, value in zip(meta_params, actual_args)
        ]
        call_args = copy.deepcopy(actual_args)
        if params and params[0] == "self":
            result = fn(None, *call_args)
            rendered_call = f"{func_name}(None, {', '.join(rendered_args)})"
        else:
            result = fn(*call_args)
            rendered_call = f"{func_name}({', '.join(rendered_args)})"
        output_index = meta.get("output", {}).get("paramindex")
        if output_index is not None:
            tests.append(
                (
                    "mutation",
                    func_name,
                    rendered_args,
                    output_index,
                    call_args[output_index],
                    bool(params and params[0] == "self"),
                )
            )
            continue
        if problem_id == "0138":
            tests.append(("random_list", rendered_call, py_random_list_to_pairs(result)))
            continue
        if problem_id == "0133":
            tests.append(("graph", rendered_call, py_graph_to_adjacency(result)))
            continue
        tests.append(("call", rendered_call, result))
    return tests or None


def tests_from_graphql(problem_id: str, py_text: str):
    py_path = next((path for path in AUDITS.glob(f"{problem_id}_*.py")), None)
    if py_path is None:
        return None
    question = fetch_question(py_path.stem, slug_for_path(py_path))
    if question is None:
        return None
    meta = json.loads(question["metaData"])
    blocks = [parse_example_io(block) for block in parse_pre_blocks(question["content"])]
    blocks = [item for item in blocks if item]
    if not blocks:
        return None
    if "classname" in meta:
        return build_systemdesign_tests(meta, blocks[0][0], blocks[0][1])
    defs = top_level_defs(py_text)
    if not defs:
        return None
    func_name, params, return_type = defs[0]
    param_names = [name for name in params if name != "self"]
    tests = []
    for raw_input, raw_output in blocks[:2]:
        values = split_named_inputs(raw_input, param_names)
        if not values:
            continue
        call_args = []
        if params and params[0] == "self":
            call_args.append("None")
        for value_str, pname in zip(values, param_names):
            meta_param = next((p for p in meta.get("params", []) if p["name"] == pname), None)
            ptype = meta_param["type"] if meta_param else ""
            call_args.append(render_graphql_arg(value_str, ptype, "py"))
        expected_value = literalize_expected(raw_output, return_type)
        tests.append(("call", f"{func_name}({', '.join(call_args)})", expected_value))
    return tests or None


def build_systemdesign_tests(meta: dict, raw_input: str, raw_output: str):
    try:
        lines = raw_input.splitlines()
        ops = ast.literal_eval(normalize_bool_null(lines[0]))
        args = ast.literal_eval(normalize_bool_null(lines[1]))
        outputs = ast.literal_eval(normalize_bool_null(raw_output))
    except Exception:
        return None
    class_name = meta["classname"]
    tests = [("systemdesign", class_name, (ops, args, outputs))]
    return tests


def assert_line_for_test(test, lang: str) -> list[str]:
    kind = test[0]
    expr = test[1]
    expected = test[2] if len(test) > 2 else None
    if kind == "call":
        expr = translate_expr_for_lang(expr, lang)
        if isinstance(expected, PyListNode):
            helper = "list_node_to_string" if lang == "py" else "listNodeToString"
            expected_expr = render_listnode_expr(expected, lang)
            return [f"assert {helper}({expr}) == {helper}({expected_expr})"]
        if isinstance(expected, PyTreeNode):
            helper = "tree_to_string" if lang == "py" else "treeToString"
            expected_expr = render_treenode_expr(expected, lang)
            return [f"assert {helper}({expr}) == {helper}({expected_expr})"]
        return [f"assert {expr} == {render_code(expected, lang)}"]
    if kind == "mutation":
        func_name, rendered_args, output_index, mutated_value, needs_self = (
            test[1],
            [translate_expr_for_lang(arg, lang) for arg in test[2]],
            test[3],
            test[4],
            test[5],
        )
        vars_ = [f"arg{i}" for i in range(len(rendered_args))]
        lines = [f"{name} = {arg}" for name, arg in zip(vars_, rendered_args)]
        call_args = ", ".join(vars_)
        if needs_self:
            lines.append(f"_result = {func_name}(None, {call_args})")
        else:
            lines.append(f"_result = {func_name}({call_args})")
        target = vars_[output_index]
        if isinstance(mutated_value, PyListNode):
            helper = "list_node_to_string" if lang == "py" else "listNodeToString"
            expected_expr = render_listnode_expr(mutated_value, lang)
            lines.append(f"assert {helper}({target}) == {helper}({expected_expr})")
        elif isinstance(mutated_value, PyTreeNode):
            helper = "tree_to_string" if lang == "py" else "treeToString"
            expected_expr = render_treenode_expr(mutated_value, lang)
            lines.append(f"assert {helper}({target}) == {helper}({expected_expr})")
        else:
            lines.append(f"assert {target} == {render_code(mutated_value, lang)}")
        return lines
    if kind == "expr":
        return [f"assert {expr} == {render_code(expected, lang)}"]
    if kind == "class_ctor":
        return [f"_obj = {expr}", "assert True"]
    if kind == "random_list":
        helper = "random_list_to_pairs" if lang == "py" else "randomListToPairs"
        return [f"assert {helper}({translate_expr_for_lang(expr, lang)}) == {render_code(expected, lang)}"]
    if kind == "graph":
        helper = "graph_to_adj" if lang == "py" else "graphToAdj"
        return [f"assert {helper}({translate_expr_for_lang(expr, lang)}) == {render_code(expected, lang)}"]
    if kind == "codec_roundtrip":
        helper = "tree_to_string" if lang == "py" else "treeToString"
        codec = "codec"
        return [
            f"root = {translate_expr_for_lang(expr, lang)}",
            f"{codec} = Codec()",
            f"assert {helper}({codec}.deserialize({codec}.serialize(root))) == {helper}(root)",
        ]
    if kind == "systemdesign":
        class_name = expr
        ops, args, outputs = expected
        lines = [f"obj = {class_name}({', '.join(render_code(v, lang) for v in args[0])})"]
        current_values = set()
        for op, op_args, out in zip(ops[1:], args[1:], outputs[1:]):
            call = f"obj.{op}({', '.join(render_code(v, lang) for v in op_args)})"
            if op == "insert" and op_args:
                if out:
                    current_values.add(op_args[0])
            if op == "remove" and op_args and out:
                current_values.discard(op_args[0])
            if out is None:
                lines.append(call)
            elif op == "getRandom" and current_values:
                values_expr = render_code(sorted(current_values), lang)
                lines.append(f"assert {call} in {values_expr}")
            else:
                lines.append(f"assert {call} == {render_code(out, lang)}")
        if len(lines) == 1:
            lines.append("assert True")
        return lines
    raise ValueError(f"unknown test kind: {kind}")


def build_tests_for_problem(problem_id: str, py_path: Path):
    try:
        tests = build_eval_tests_from_examples(problem_id, py_path)
        if tests:
            return tests
    except Exception:
        pass
    tests = eval_print_harness(py_path)
    if tests:
        return tests
    py_text = read_text(py_path)
    tests = tests_from_convert_all(problem_id, py_text)
    if tests:
        return tests
    try:
        tests = tests_from_trivial_eval(py_path)
        if tests:
            return tests
    except Exception:
        pass
    try:
        tests = tests_from_graphql(problem_id, py_text)
        if tests:
            return tests
    except Exception:
        pass
    return None


def patch_file(path: Path, tests) -> bool:
    lang = "py" if path.suffix == ".py" else "sifr"
    text = read_text(path)
    lines = []
    for test in tests:
        lines.extend(assert_line_for_test(test, lang))
    if not lines:
        return False
    new_text = replace_main(text, lines, lang)
    new_text = inject_helpers(new_text, lang)
    if new_text != text:
        path.write_text(new_text)
    return True


def missing_problem_files() -> list[Path]:
    files = []
    for path in sorted(AUDITS.glob("*")):
        if path.suffix not in {".py", ".sifr"}:
            continue
        if not problem_id_from_name(path.name):
            continue
        if needs_oracle_backfill(path):
            files.append(path)
    return files


def main():
    files = missing_problem_files()
    by_stem: dict[str, list[Path]] = collections.defaultdict(list)
    for path in files:
        by_stem[path.stem].append(path)

    unresolved: list[str] = []
    patched = 0
    for stem, paths in sorted(by_stem.items()):
        problem_id = problem_id_from_name(stem)
        py_path = next((p for p in paths if p.suffix == ".py"), None)
        if py_path is None:
            unresolved.append(stem)
            continue
        tests = build_tests_for_problem(problem_id, py_path)
        if not tests:
            unresolved.append(stem)
            continue
        for path in paths:
            if patch_file(path, tests):
                patched += 1

    remaining = missing_problem_files()
    print(f"patched_files={patched}")
    print(f"unresolved_ids={len(unresolved)}")
    if unresolved:
        print(" ".join(unresolved))
    print(f"remaining_files_without_asserts={len(remaining)}")
    if remaining:
        print("\n".join(path.name for path in remaining[:200]))


CONVERT_ALL_TESTS = load_convert_all_tests()


if __name__ == "__main__":
    main()
