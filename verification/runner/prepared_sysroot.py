"""Isolated entry point for source-bound sysroot preparation artifacts."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from sifr_verify.prepared_sysroot import main
from sifr_verify.errors import VerificationError

if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except VerificationError as error:
        print(f"sysroot-preparation: infrastructure={getattr(error, 'classification', 'unavailable')} {error}", file=sys.stderr)
        raise SystemExit(2)
