from __future__ import annotations

import random

from schwifty import BBAN


def main() -> int:
    random.seed(25)
    generated = [BBAN.random("DE") for _ in range(16)]
    if not all(bban.validate_national_checksum() for bban in generated):
        raise RuntimeError(
            "Schwifty generated a BBAN with an invalid national checksum"
        )
    print("python minor train features ok: schwifty-bban-checksums=16")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
