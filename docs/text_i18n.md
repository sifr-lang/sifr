# Text, Unicode, Encoding, And I18n

Sifr text is valid Unicode scalar text backed by Rust `String`/`str` invariants. Arbitrary bytes stay `bytes` until an explicit decode boundary, and text becomes bytes only through an explicit encode boundary.

## Encoding

Use `sifr.encoding` for byte/text conversion:

```python
from sifr.encoding import ascii, latin1, decode, encode, replace_decode_handler

try:
    data: bytes = encode("caf\u00e9", latin1())
    text: str = decode(data, latin1())
    recovered: str = decode(b"\xffA", ascii(), replace_decode_handler())
except Error as e:
    _ = e.message
```

Supported phase-exit encodings are Tier 0 (`utf-8`, `utf-8-sig`, `ascii`, `latin-1`, `utf-16-le`, `utf-16-be`) and selected Tier 1 Windows-125x labels through `encoding_rs`, including `windows-1252` / `cp1252`. Tier 2 CJK and UTF-32 encodings are deferred.

Encoding failures return typed `DecodeError` or `EncodeError`. Recovery-capable APIs return `DecodeOutcome` or `EncodeOutcome` so recovery diagnostics are not discarded. Error handlers are typed values; dynamic handler registration and dynamic handler names are unsupported.

## Explicit Text I/O

Text file I/O requires an explicit encoding:

```python
from sifr.encoding import latin1
from sifr.io import open_text

try:
    writer = open_text("/tmp/example.txt", "w", encoding=latin1())
    _ = writer.write("caf\u00e9")
    writer.close()
except IOError as e:
    _ = e.message
```

The builtin `open(..., encoding=..., errors=...)` lowers to the same text I/O substrate. `open(path)` and text-mode `open(path, "r")` without `encoding=` are intentionally unsupported; Sifr never uses locale-derived default text encodings.

## Unicode

Use `sifr.unicode` for Unicode data and segmentation:

```python
from sifr.unicode import NFC, category, graphemes, name, normalize

try:
    composed: str = normalize(NFC, "e\u0301")
    snowman: str = name("\u2603")
    kind: str = category("A")
    clusters: list[str] = graphemes("a\u0301b")
except UnicodeDataError as e:
    _ = e.message
```

The phase ships Unicode 17.0.0 data for normalization, names, scalar properties, numeric values, case folding, grapheme boundaries, and word boundaries. Sentence boundaries and streaming segmentation cursors are deferred.

## Locale And I18n

Use `sifr.i18n` with explicit locale objects and formatter objects:

```python
from sifr.i18n import LocaleId, NumberFormatter, PluralRules, PLURAL_CARDINAL

locale = LocaleId("en")
try:
    formatted: str = NumberFormatter(locale).format("1000")
    category: str = PluralRules(locale, PLURAL_CARDINAL).category("2")
except Error as e:
    _ = e.message
```

`host_locale()` is read-only and host-limited. It cannot make implicit text encodings legal.

Translations use explicit bundles and fallback chains:

```python
from sifr.i18n import bundle_from_mo_bytes, translator

try:
    primary = bundle_from_mo_bytes(catalog_bytes)
    tx = translator(primary)
    text: str = tx.translate_plural("file", "files", 2)
except Error as e:
    _ = e.message
```

`.mo` files are a compatibility backend behind the native `Bundle` / `Translator` API. Catalog parsing is deterministic, uses the encoding substrate for declared charsets, and rejects unsupported plural expressions with `CatalogError`.

## Intentional Python-Shaped Differences

The production API centers are `sifr.encoding`, `sifr.unicode`, `sifr.io`, and `sifr.i18n`.

Not production APIs in this phase:

- `sifr.codecs`
- `sifr.encodings`
- `sifr.unicodedata`
- `sifr.locale`
- `sifr.gettext`
- bare `codecs`, `encodings`, `unicodedata`, `locale`, or `gettext` imports
- `codecs.register`, `codecs.register_error`, `locale.setlocale`, `gettext.install`, global `_`

Future Python-shaped adapters must be reviewed separately and must wrap the native Sifr substrate without process-global mutation.
