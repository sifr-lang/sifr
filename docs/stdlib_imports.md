# Standard Library Imports

Sifr standard library modules are public under the `sifr.*` namespace. Use explicit imports from `sifr.<module>`:

```python
from sifr.math import sqrt
from sifr.collections import deque
from sifr.json import dumps
from sifr.encoding import decode
from sifr.unicode import normalize
from sifr.i18n import LocaleId
```

Bare CPython stdlib module names such as `math`, `json`, `os`, `heapq`, `collections`, `codecs`, `encodings`, `unicodedata`, `locale`, and `gettext` are not aliases for Sifr stdlib modules. Sifr uses Python syntax and follows CPython behavior where that fits the safety model, but Sifr source is not Python-source-compatible.

If a real user or package module named `math`, `json`, or similar exists, normal top-level resolution can import it. If no real top-level module exists, a bare stdlib import is rejected with `SIFR-IMPORT-0008` and a `sifr.*` suggestion.

Unsupported:

```python
from math import sqrt
import json
```

Supported:

```python
from sifr.math import sqrt
from sifr.json import dumps
```

`import sifr.math` is also unsupported for now because Sifr does not yet support module-object imports. Import the symbols you need from the module instead.

Text, Unicode, encoding, and i18n surfaces are documented in [text_i18n.md](./text_i18n.md).
