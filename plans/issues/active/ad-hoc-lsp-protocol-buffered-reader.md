# LSP verification client: buffered-frame timeout and failure preservation

status: open
owner: developer-tooling verification / protocol transport
blocks: DX.1 demanded-stdlib LSP baseline and therefore DX.2–DX.16 sequencing
discovered: 2026-09-16, draft PR #3840
implementation candidate: `28605fd6da80d8eaf877cd6a992e1e8ab4b6d69f`

## Problem and ownership

The existing `verification/areas/developer_tooling/lsp_protocol.py` creates a
buffered subprocess stdout reader (line 42), waits on the underlying file
descriptor with `select.select` (line 170), then uses buffered `read` calls
(lines 175 and 186). One read can buffer a second complete LSP frame. A later
`select` then times out because the kernel pipe is empty even though Python
already holds the next message. This file is unchanged from the phase base
`0f819c2f04bf5b2891074c55ba26369ddf4f13bd`; no DX.1 compiler change caused it.

The DX.1 installed optimized baseline passed warmup plus 20 CLI checks and
native first/no-op/semantic-edit builds, then timed out on `workspace/diagnostic`
after `didClose`. The trace records the close notification and its published
diagnostics but no received response to request 4 before the 90-second deadline.
The transport defect independently reproduces without running Sifr. It invalidates
that protocol timing result; the trace alone does not establish a compiler stall.

Cleanup raised `LSP exited 1` over the original timeout. The full traceback keeps
both, but future protocol handling must preserve the primary failure explicitly.
The user authorized correcting this required measurement dependency in the
DX.1 batch, under the same owned checkout and review. Do not remove the retention operation, increase timeouts, weaken outcomes,
substitute an alternate client, or call the failed run a performance pass.

## Minimal independent reproduction

Run from the repository on the remote host. It writes two complete frames in
one OS write and keeps the producer alive. The second frame is already buffered
when the second read incorrectly waits on the OS descriptor.

```python
import json, os, shlex, sys, time
sys.path.insert(0, "verification/areas/developer_tooling")
from lsp_protocol import LspClient, LspProtocolError
body = json.dumps({"jsonrpc": "2.0", "method": "fixture/notification", "params": {}}).encode()
frame = f"Content-Length: {len(body)}\r\n\r\n".encode() + body
code = f"import os,time; os.write(1,{(frame + frame)!r}); time.sleep(10)"
os.environ["SIFR_LSP_COMMAND"] = shlex.join([sys.executable, "-c", code])
client = LspClient(timeout=1)
try:
    print("first:", client._read_message(time.monotonic() + 1))
    try:
        print("second:", client._read_message(time.monotonic() + 1))
    except LspProtocolError as error:
        print("second read error:", str(error))
        print("buffered bytes still available:", len(client.process.stdout.peek(1)))
finally:
    client.process.terminate()
    client.process.wait(timeout=5)
```

Observed: first frame decoded, second read timed out, **88 bytes** remained in
`BufferedReader`. The child was terminated and reaped. This was a bounded
independent diagnostic, not a rerun of the failed performance sample.

## Evidence

All raw evidence remains outside the reviewed tree on
`yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx1-evidence/`:

| Artifact | SHA-256 |
| --- | --- |
| `lsp-buffer-repro.txt` | `b191722bfb56b49277c809e68898dcdb23192cfe2902a0ecc8115dd77add9299` |
| `product-baseline-explicit-toolchain-28605fd/baseline.json` | `b27c216248e0d757723fb603c739c6deffed40385bbcfd2d63ca0177e105ec7f` |
| `product-baseline-explicit-toolchain-28605fd.log` | `436c3c521e321c44b92d1f5090103512cde84ffa3bde948f935b559191d83f3b` |

Baseline command:

```bash
RUSTUP_TOOLCHAIN=1.98.1-x86_64-unknown-linux-gnu CARGO_BUILD_JOBS=2 \
  python3 verification/areas/performance/dx_capture.py \
  --lane product-installed-optimized \
  --receipt /home/yaser5/projects/sifr/dx1-evidence/product-28605fd/receipt.json \
  --output /home/yaser5/projects/sifr/dx1-evidence/product-baseline-explicit-toolchain-28605fd
```

## Required correction and resumption boundary

This DX.1 batch must implement deadline-aware framing that handles queued/buffered,
coalesced and fragmented messages without waiting on an empty OS descriptor or
blocking indefinitely on a partial frame. Add independent transport regression
coverage for those cases and preserve the primary error through cleanup.
Then resume the [DX.1 handoff](ad-hoc-compiler-dx-and-toolchain-reuse.md): revalidate
the affected protocol workload with the repaired canonical client, complete the
missing baselines and scoped review, and only then merge. DX.1 is not closed by
this issue record. No DX.2–DX.16 implementation is authorized by this handoff.

Named regression command: `python3 verification/areas/developer_tooling/test_lsp_protocol_transport.py`. Covers coalesced frames, fragmented headers/bodies, partial-frame deadlines, EOF, queued frames after exit, and primary failure preservation through cleanup.
