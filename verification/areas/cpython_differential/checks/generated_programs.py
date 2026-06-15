"""Deterministic CPython differential program generator and shrinker."""

from __future__ import annotations

import random
from dataclasses import dataclass
from typing import Any

SUPPORTED_SHAPES = {
    "arith_branch",
    "string_choice",
    "list_tuple_loop",
    "dict_sorted",
}


@dataclass(frozen=True)
class GeneratedProgram:
    python_source: str
    sifr_source: str
    grammar_shape: str
    parameters: dict[str, Any]


def generate_program(case: dict[str, Any]) -> GeneratedProgram:
    shape = str(case["shape"])
    if shape not in SUPPORTED_SHAPES:
        raise ValueError(f"unsupported generated-program shape {shape!r}")
    rng = random.Random(int(case["seed"]))
    if shape == "arith_branch":
        return generate_arith_branch(rng)
    if shape == "string_choice":
        return generate_string_choice(rng)
    if shape == "list_tuple_loop":
        return generate_list_tuple_loop(rng)
    return generate_dict_sorted(rng)


def generate_arith_branch(rng: random.Random) -> GeneratedProgram:
    base = rng.randint(12, 80)
    threshold = rng.randint(8, 72)
    addend = rng.randint(2, 17)
    subtrahend = rng.randint(1, 9)
    multiplier = rng.randint(2, 5)
    divisor = rng.randint(2, 9)
    modulo = rng.randint(2, 11)
    python_source = f"""import json


def adjust(value: int, flag: bool) -> int:
    if flag:
        return value + {addend}
    else:
        return value - {subtrahend}


def main() -> None:
    base = {base}
    threshold = {threshold}
    flag = base > threshold
    scaled = adjust(base, flag) * {multiplier}
    values = [adjust(base, flag), scaled // {divisor}, scaled % {modulo}]
    print(json.dumps(values, separators=(",", ":"), sort_keys=True))


if __name__ == "__main__":
    main()
"""
    sifr_source = f"""from sifr.json import json_dumps


def adjust(value: int, flag: bool) -> int:
    if flag:
        return value + {addend}
    else:
        return value - {subtrahend}


def main() -> None:
    base: int = {base}
    threshold: int = {threshold}
    flag: bool = base > threshold
    scaled: int = adjust(base, flag) * {multiplier}
    values: list[int] = [adjust(base, flag), scaled // {divisor}, scaled % {modulo}]
    print(json_dumps(values))
"""
    return GeneratedProgram(
        python_source=python_source,
        sifr_source=sifr_source,
        grammar_shape="arith_branch",
        parameters={
            "base": base,
            "threshold": threshold,
            "addend": addend,
            "subtrahend": subtrahend,
            "multiplier": multiplier,
            "divisor": divisor,
            "modulo": modulo,
        },
    )


def generate_string_choice(rng: random.Random) -> GeneratedProgram:
    left = rng.choice(["sifr", "rust", "safe"])
    right = rng.choice(["python", "native", "typed"])
    suffix = rng.choice(["-ok", "-checked", "-green"])
    probe = left if rng.choice([True, False]) else right
    python_source = f"""import json


def choose(flag: bool, left: str, right: str) -> str:
    if flag:
        return left + "{suffix}"
    else:
        return right + "{suffix}"


def main() -> None:
    left = "{left}"
    right = "{right}"
    probe = "{probe}"
    values = [choose(probe == left, left, right), left + ":" + right]
    print(json.dumps(values, separators=(",", ":"), sort_keys=True))


if __name__ == "__main__":
    main()
"""
    sifr_source = f"""from sifr.json import json_dumps


def choose(flag: bool, left: str, right: str) -> str:
    if flag:
        return left + "{suffix}"
    else:
        return right + "{suffix}"


def main() -> None:
    left: str = "{left}"
    right: str = "{right}"
    probe: str = "{probe}"
    values: list[str] = [choose(probe == left, left, right), left + ":" + right]
    print(json_dumps(values))
"""
    return GeneratedProgram(
        python_source=python_source,
        sifr_source=sifr_source,
        grammar_shape="string_choice",
        parameters={"left": left, "right": right, "suffix": suffix, "probe": probe},
    )


def generate_list_tuple_loop(rng: random.Random) -> GeneratedProgram:
    first = rng.randint(1, 9)
    second = rng.randint(1, 9)
    items = [rng.randint(1, 9) for _ in range(4)]
    items_literal = ", ".join(str(item) for item in items)
    python_source = f"""import json


def main() -> None:
    pair = ({first}, {second})
    items = [{items_literal}]
    total = 0
    for value in items:
        total = total + value
    values = [pair[0], pair[1], total, len(items)]
    print(json.dumps(values, separators=(",", ":"), sort_keys=True))


if __name__ == "__main__":
    main()
"""
    sifr_source = f"""from sifr.json import json_dumps


def main() -> None:
    pair: tuple[int, int] = ({first}, {second})
    items: list[int] = [{items_literal}]
    total: int = 0
    for value in items:
        total = total + value
    values: list[int] = [pair[0], pair[1], total, len(items)]
    print(json_dumps(values))
"""
    return GeneratedProgram(
        python_source=python_source,
        sifr_source=sifr_source,
        grammar_shape="list_tuple_loop",
        parameters={"pair": [first, second], "items": items},
    )


def generate_dict_sorted(rng: random.Random) -> GeneratedProgram:
    alpha = rng.randint(1, 20)
    beta = rng.randint(1, 20)
    gamma = rng.randint(1, 20)
    python_source = f"""import json


def main() -> None:
    values = {{"alpha": {alpha}, "beta": {beta}, "gamma": {gamma}}}
    result = [
        values["alpha"] == {alpha},
        values["beta"] == {beta},
        values["gamma"] == {gamma},
        len(values) == 3,
    ]
    print(json.dumps(result, separators=(",", ":"), sort_keys=True))


if __name__ == "__main__":
    main()
"""
    sifr_source = f"""from sifr.json import json_dumps


def main() -> None:
    values: dict[str, int] = {{"alpha": {alpha}, "beta": {beta}, "gamma": {gamma}}}
    result: list[bool] = [
        values["alpha"] == {alpha},
        values["beta"] == {beta},
        values["gamma"] == {gamma},
        len(values) == 3,
    ]
    print(json_dumps(result))
"""
    return GeneratedProgram(
        python_source=python_source,
        sifr_source=sifr_source,
        grammar_shape="dict_sorted",
        parameters={"alpha": alpha, "beta": beta, "gamma": gamma},
    )


def minimized_candidate(case: dict[str, Any]) -> GeneratedProgram:
    shape = str(case["shape"])
    minimal_case = {"seed": 1, "shape": shape}
    if shape == "arith_branch":
        minimal_case["seed"] = 1001
        return GeneratedProgram(
            python_source="""import json


def adjust(value: int, flag: bool) -> int:
    if flag:
        return value + 1
    else:
        return value - 1


def main() -> None:
    values = [adjust(2, True)]
    print(json.dumps(values, separators=(",", ":"), sort_keys=True))


if __name__ == "__main__":
    main()
""",
            sifr_source="""from sifr.json import json_dumps


def adjust(value: int, flag: bool) -> int:
    if flag:
        return value + 1
    else:
        return value - 1


def main() -> None:
    values: list[int] = [adjust(2, True)]
    print(json_dumps(values))
""",
            grammar_shape=shape,
            parameters={"minimized": True},
        )
    if shape == "string_choice":
        return generate_program({"seed": 1, "shape": "string_choice"})
    if shape == "list_tuple_loop":
        return generate_program({"seed": 1, "shape": "list_tuple_loop"})
    return generate_program({"seed": 1, "shape": "dict_sorted"})
