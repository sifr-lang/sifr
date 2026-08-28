#!/usr/bin/env python3
"""Verification hardening runner entrypoint."""

from . import main
from ..profile_commands import install_terminal_signal_handlers


if __name__ == "__main__":
    install_terminal_signal_handlers()
    raise SystemExit(main())
