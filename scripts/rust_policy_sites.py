"""Small Rust lexical scanner for review inventories, never a safety proof.

No compiler, Cargo cache, Git index or third-party Python package is required.
Comments and character/string literals are separate tokens. Lifetimes remain code.
Fingerprints bind complete token spans, including literal contents but excluding
comments and whitespace; source offsets serve diagnostics only.
"""
from __future__ import annotations

from collections import Counter
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import re


@dataclass(frozen=True)
class Token:
    kind: str
    value: str
    start: int
    end: int


@dataclass(frozen=True)
class Site:
    path: str
    scope: str
    kind: str
    ordinal: int
    line: int
    fingerprint: str
    text: str
    start: int
    end: int
    test_only: bool = False
    source_evidence: str = ""
    context_text: str = ""

    @property
    def key(self) -> str:
        return f"{self.path}::{self.scope}::{self.kind}::{self.ordinal}"

    def record(self) -> dict:
        return {"site": self.key, "fingerprint": self.fingerprint}


_RAW = re.compile(r'(?:br|cr|r)(#*)"')
_STRING = re.compile(r'(?:b|c)?"')
_CHAR = re.compile(r"b?'(?:[^'\\\n]|\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|[^\n]))'")
_IDENT = re.compile(r"(?:r#)?[A-Za-z_][A-Za-z_0-9]*")


def tokens(source: str) -> list[Token]:
    result = []
    i = 0
    while i < len(source):
        start = i
        if source[i].isspace():
            i += 1
            continue
        if source.startswith('//', i):
            end = source.find('\n', i)
            i = len(source) if end < 0 else end
            result.append(Token('comment', source[start:i], start, i))
            continue
        if source.startswith('/*', i):
            depth = 1
            i += 2
            while i < len(source) and depth:
                if source.startswith('/*', i):
                    depth += 1
                    i += 2
                elif source.startswith('*/', i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            if depth:
                raise ValueError(f'unterminated block comment at {start}')
            result.append(Token('comment', source[start:i], start, i))
            continue
        raw = _RAW.match(source, i)
        if raw:
            delimiter = '"' + raw[1]
            end = source.find(delimiter, i + len(raw[0]))
            if end < 0:
                raise ValueError(f'unterminated raw string at {start}')
            i = end + len(delimiter)
            result.append(Token('string', source[start:i], start, i))
            continue
        prefix = _STRING.match(source, i)
        if prefix:
            i += len(prefix[0])
            while i < len(source):
                if source[i] == '\\':
                    i += 2
                elif source[i] == '"':
                    i += 1
                    break
                else:
                    i += 1
            else:
                raise ValueError(f'unterminated string at {start}')
            result.append(Token('string', source[start:i], start, i))
            continue
        # A character is exactly one scalar or one escaped scalar followed by a
        # closing quote. An identifier after apostrophe is a Rust lifetime/label.
        char = _CHAR.match(source, i)
        if char:
            i += len(char[0])
            result.append(Token('char', source[start:i], start, i))
            continue
        ident = _IDENT.match(source, i)
        if ident:
            i += len(ident[0])
            result.append(Token('ident', source[start:i], start, i))
            continue
        operator = next((op for op in ('::', '=>', '==', '!=', '&&', '||', '->', '..=', '..')
                         if source.startswith(op, i)), source[i])
        i += len(operator)
        result.append(Token('punct', operator, start, i))
    return result


def code_tokens(source: str) -> list[Token]:
    return [t for t in tokens(source) if t.kind != 'comment']


def pairs(ts: list[Token]) -> dict[int, int]:
    stack = []
    result = {}
    for i, t in enumerate(ts):
        if t.kind != 'punct':
            continue
        if t.value in ('(', '[', '{'):
            stack.append(i)
        elif t.value in (')', ']', '}'):
            if not stack or ts[stack[-1]].value != {')': '(', ']': '[', '}': '{'}[t.value]:
                raise ValueError(f'unbalanced delimiter at {t.start}')
            left = stack.pop()
            result[left] = i
            result[i] = left
    if stack:
        raise ValueError(f'unclosed delimiter at {ts[stack[-1]].start}')
    return result


def fingerprint(ts: list[Token]) -> str:
    normalized = [(t.kind, t.value) for t in ts]
    return hashlib.sha256(json.dumps(normalized, ensure_ascii=False,
                                     separators=(',', ':')).encode()).hexdigest()


def item_header_end(ts: list[Token], ps: dict[int, int], start: int) -> int:
    """Find the item body/terminator outside grouped and generic header tokens."""
    j, angles = start, 0
    while j < len(ts):
        value = ts[j].value
        if value in ('(', '[') or (value == '{' and angles):
            j = ps[j] + 1
            continue
        if value == '<':
            angles += 1
        elif value == '>' and angles:
            angles -= 1
        elif not angles and value in ('{', ';', '='):
            break
        j += 1
    return j


class Source:
    def __init__(self, path: str, text: str):
        self.path, self.text = path, text
        self.comments = [t for t in tokens(text) if t.kind == "comment"]
        self.ts = code_tokens(text)
        self.pairs = pairs(self.ts)
        self.functions = []
        seen = Counter()
        for i, t in enumerate(self.ts[:-2]):
            if t.value != 'fn' or self.ts[i + 1].kind != 'ident':
                continue
            name = self.ts[i + 1].value
            j = item_header_end(self.ts, self.pairs, i + 2)
            if j < len(self.ts) and self.ts[j].value == '{':
                seen[name] += 1
                scope = name if seen[name] == 1 else f'{name}#{seen[name]}'
                declaration_start = i
                while declaration_start and (self.ts[declaration_start - 1].value
                        in ('unsafe', 'extern', 'const', 'async')
                        or self.ts[declaration_start - 1].kind == 'string'):
                    declaration_start -= 1
                self.functions.append((declaration_start, j, self.pairs[j], scope))
        self.contexts = list(self.functions)
        for i, t in enumerate(self.ts[:-3]):
            if (t.value == 'macro_rules' and self.ts[i + 1].value == '!'
                    and self.ts[i + 2].kind == 'ident'
                    and self.ts[i + 3].value in ('{', '(', '[')):
                opening = i + 3
                name = 'macro_rules!::' + self.ts[i + 2].value
                self.contexts.append((i, opening, self.pairs[opening], name))
        self.counts = Counter()
        self.context_hashes = {}

    def scope(self, i: int) -> str:
        enclosing = [(a, b, c, name) for a, b, c, name in self.contexts if a <= i <= c]
        return max(enclosing, default=(0, 0, 0, '<module>'), key=lambda x: x[0])[3]

    def site(self, kind: str, start: int, end: int) -> Site:
        scope = self.scope(start)
        self.counts[(scope, kind)] += 1
        a, b = self.ts[start].start, self.ts[end].end
        enclosing = [(a, b, c, name) for a, b, c, name in self.contexts if a <= start <= c]
        context_text = ""
        if enclosing:
            first, _body, last, _name = max(enclosing, key=lambda x: x[0])
            context_text = self.text[self.ts[first].start:self.ts[last].end]
        return Site(self.path, scope, kind, self.counts[(scope, kind)],
                    self.text.count('\n', 0, a) + 1,
                    self.site_fingerprint(start, end), self.text[a:b], a, b,
                    self.test_only(start), self.safety_evidence(a, b), context_text)

    def safety_evidence(self, start: int, end: int) -> str:
        """Bind complete adjacent comment runs, never a distance-based neighbor.

        Expression prefixes (let result =, Ok(...)) may intervene. Statement or
        block boundaries terminate adjacency. Nested operations own their own
        comments; a declaration does not inherit its body's SAFETY comments.
        """
        preceding = [t for t in self.comments if t.end <= start]
        if not preceding:
            return ''
        last = preceding[-1]
        gap = self.text[last.end:start]
        # A trailing comment already belongs to code on its own line.
        line_start = self.text.rfind('\n', 0, last.start) + 1
        if self.text[line_start:last.start].strip():
            return ''
        depth = 0
        for token in code_tokens(gap):
            if token.kind != 'punct':
                continue
            if token.value in (';', '{', '}', '=>') or (token.value == ',' and not depth):
                return ''
            if token.value in ('(', '['):
                depth += 1
            elif token.value in (')', ']'):
                depth = max(0, depth - 1)
        # A blank line separates a neighboring explanation from this operation.
        if re.search(r'\n[ \t]*\n', gap):
            return ''
        run = [last]
        for comment in reversed(preceding[:-1]):
            gap = self.text[comment.end:run[0].start]
            if gap.strip() or re.search(r'\n[ \t]*\n', gap):
                break
            # Every comment must own its physical line. A continuation after
            # another statement's trailing marker cannot inherit that marker.
            line_start = self.text.rfind('\n', 0, comment.start) + 1
            if self.text[line_start:comment.start].strip():
                break
            run.insert(0, comment)
        marker = next((i for i, t in enumerate(run)
                       if re.search(r'(?:\bSAFETY:|# Safety\b)', t.value)), None)
        if marker is None:
            return ''
        return '\n'.join(t.value for t in run[marker:])

    def site_fingerprint(self, start: int, end: int) -> str:
        # Selector-only fingerprints would miss changed branch bodies, and an
        # isolated unsafe block would miss removed admission/bounds validation.
        # Bind the lexical site and its enclosing function's complete context.
        enclosing = [(a, b, c, name) for a, b, c, name in self.contexts if a <= start <= c]
        site_hash = fingerprint(self.ts[start:end + 1])
        if not enclosing:
            return site_hash
        a, _b, c, _name = max(enclosing, key=lambda x: x[0])
        if (a, c) not in self.context_hashes:
            self.context_hashes[(a, c)] = fingerprint(self.ts[a:c + 1])
        context_hash = self.context_hashes[(a, c)]
        return hashlib.sha256((site_hash + ':' + context_hash).encode()).hexdigest()

    def test_only(self, i: int) -> bool:
        # A test module allowance is narrow only if its source cfg(test) guard
        # is adjacent. A name containing "test" is not evidence of isolation.
        j = i - 1
        while j >= 0 and self.ts[j].value == ']':
            opening = self.pairs[j]
            values = [t.value for t in self.ts[opening + 1:j]]
            if values == ['cfg', '(', 'test', ')']:
                return True
            j = opening - 2  # skip preceding # token
        return False


def rust_sources(root: Path):
    # All first-party crate source/test/build files, excluding bundled native
    # inputs. New crates and files are discovered in a cold checkout.
    for path in sorted((root / 'crates').rglob('*.rs')):
        rel = path.relative_to(root).as_posix()
        if any(p in ('target', 'third_party', 'vendor') for p in path.relative_to(root).parts):
            continue
        yield Source(rel, path.read_text())


def reconcile(sites: list[Site], records: list[dict]) -> list[str]:
    errors = []
    current = {s.key: s for s in sites}
    recorded = {}
    for record in records:
        key = record.get('site')
        if not isinstance(key, str) or key in recorded:
            errors.append(f'duplicate or invalid site: {key}')
            continue
        recorded[key] = record
    for key, site in current.items():
        if key not in recorded:
            errors.append(f'{site.path}:{site.line}: new/unclassified {key}')
        elif recorded[key].get('fingerprint') != site.fingerprint:
            errors.append(f'{site.path}:{site.line}: changed fingerprint {key}')
    for key in recorded.keys() - current.keys():
        errors.append(f'stale site: {key}')
    return errors


def read_inventory(root: Path, name: str) -> dict:
    data = json.loads((root / 'verification/policy' / name).read_text())
    if data.get('schema_version') != 1 or not isinstance(data.get('sites'), list):
        raise ValueError(f'invalid inventory schema: {name}')
    return data
