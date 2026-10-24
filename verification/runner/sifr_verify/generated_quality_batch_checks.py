"""Enroll fresh generated-corpus ownership checks in the runner foundation gate."""
import io
import unittest
from contextlib import redirect_stderr, redirect_stdout

from .generated_cargo_setup import quality_module


def policy_checks():
    quality_module()
    import test_fresh_corpus
    stream = io.StringIO()
    with redirect_stdout(stream), redirect_stderr(stream):
        result = unittest.TextTestRunner(stream=stream).run(
            unittest.defaultTestLoader.loadTestsFromModule(test_fresh_corpus))
    if not result.wasSuccessful():
        raise AssertionError(stream.getvalue())
    print(f"fresh generated quality batch checks passed ({result.testsRun} tests)")
