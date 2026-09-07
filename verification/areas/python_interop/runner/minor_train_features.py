from __future__ import annotations

import random

from schwifty import BBAN

from dependency_versions import runtime_version_marker


def main() -> int:
    versions = runtime_version_marker("schwifty")
    random.seed(25)
    country_codes = ("BE", "DE", "ES", "FR", "IT", "NL", "NO", "PL")
    generated = [
        BBAN.random(country_code) for country_code in country_codes for _ in range(4)
    ]
    if not all(bban.validate_national_checksum() for bban in generated):
        raise RuntimeError(
            "Schwifty generated a BBAN with an invalid national checksum"
        )
    print(
        f"python minor train features ok: {versions} "
        f"countries={len(country_codes)} schwifty-bban-checksums={len(generated)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
