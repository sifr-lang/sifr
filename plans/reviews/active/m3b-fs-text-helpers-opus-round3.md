Round-2 blocking finding is resolved. Retained intrinsics no longer expose `read_text`/`write_text`/`exists`/`read_lines`/`append_text`; the codegen registry test asserts these lower via `_sifr.fs` declarations; `stdlib/_sifr/fs.sifr` carries the `@rust(sifr_stdlib.fs.*)` bindings; `_sifr/io.sifr` is empty as intended.

READY
