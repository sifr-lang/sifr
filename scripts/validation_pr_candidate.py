"""Bind an open PR's synthetic merge to its exact base and head objects."""
from pathlib import Path
import re
import subprocess


def verify_pr_candidate(repo: Path, candidate: str, base: str, head: str) -> None:
    if any(not isinstance(value, str) or not re.fullmatch(r'[0-9a-f]{40}', value)
           for value in (candidate, base, head)):
        raise ValueError('PR candidate, base and head must be complete commit SHAs')
    try:
        def read(*args):
            return subprocess.check_output(
                ['git', '-C', str(repo), '--no-replace-objects', 'cat-file', *args],
                stderr=subprocess.PIPE, timeout=30)
        if read('-t', candidate).strip() != b'commit':
            raise ValueError('PR candidate must be a commit object')
        # Read stored headers, not rev-list/show's possibly grafted or shallow
        # history. GitHub's open-PR test merge has base first and head second;
        # merged PR squash/rebase commits and merge queues are different inputs.
        headers = read('commit', candidate).split(b'\n\n', 1)[0].splitlines()
        parents = [line[7:] for line in headers if line.startswith(b'parent ')]
    except (OSError, subprocess.SubprocessError) as error:
        raise ValueError('PR candidate commit object is unavailable') from error
    if parents != [base.encode('ascii'), head.encode('ascii')]:
        raise ValueError('PR merge parents differ from the exact required base/head')
