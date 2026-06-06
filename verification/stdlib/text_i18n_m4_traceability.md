# Text/I18n M4 Traceability

Milestone: `milestone_text_i18n_4`

| Backlog item | Required fixture/evidence |
| --- | --- |
| `Bundle`, `Message`, `Translator` | Native lookup, fallback, missing-key, context, and plural fixtures. |
| `.mo` loader | Little-endian/big-endian, declared charset, malformed magic/version/table fixtures mined from CPython. |
| Safe plural parser | Supported operators, unsupported syntax rejection, no general expression engine. |
| M1 substrate reuse | `.mo` declared encoding decoding routes through `sifr.encoding`; no local decoder. |
| Unsupported gettext globals | Negative fixtures or inventory entries for `gettext.install`, global `_`, bind/textdomain mutation. |
| Future backends | Fluent and ICU message-format backends deferred without API commitment. |
