#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DOCS_DIR="$REPO_ROOT/docs"
SOURCE_DIR="${1:-$REPO_ROOT/.mintlify-import}"
PRESERVE_DIR="$REPO_ROOT/.docs-preserve"

if [[ ! -d "$SOURCE_DIR" ]]; then
  echo "error: Mintlify export not found at: $SOURCE_DIR" >&2
  echo "Copy it first, for example:" >&2
  echo "  cp -R ~/Downloads/mintlify-community-sifr \"$REPO_ROOT/.mintlify-import\"" >&2
  echo "  $0" >&2
  exit 1
fi

if [[ ! -f "$SOURCE_DIR/docs.json" ]]; then
  echo "error: expected docs.json in $SOURCE_DIR" >&2
  exit 1
fi

echo "Importing Mintlify docs from: $SOURCE_DIR"
echo "Target: $DOCS_DIR"

mkdir -p "$PRESERVE_DIR"
rm -rf "$PRESERVE_DIR"/*

preserve_path() {
  local rel="$1"
  if [[ -e "$DOCS_DIR/$rel" ]]; then
    echo "  preserving $rel"
    mkdir -p "$(dirname "$PRESERVE_DIR/$rel")"
    mv "$DOCS_DIR/$rel" "$PRESERVE_DIR/$rel"
  fi
}

echo "Backing up internal reference docs..."
preserve_path errors
preserve_path schemas
preserve_path cli_command_semantics.md
preserve_path concurrency_runtime.md
preserve_path formatter.md
preserve_path linter.md
preserve_path network_http.md
preserve_path package_management.md
preserve_path self_update.md
preserve_path stdlib_imports.md
preserve_path text_i18n.md

echo "Removing previous Mintlify scaffold..."
rm -f "$DOCS_DIR/docs.json" "$DOCS_DIR/style.css"
rm -f "$DOCS_DIR/index.mdx" "$DOCS_DIR/installation.mdx" "$DOCS_DIR/quickstart.mdx"
rm -rf "$DOCS_DIR/language" "$DOCS_DIR/cli" "$DOCS_DIR/guides" "$DOCS_DIR/logo"

echo "Copying Mintlify export..."
rsync -a \
  --exclude '.git' \
  --exclude '.github' \
  --exclude 'node_modules' \
  --exclude '.DS_Store' \
  "$SOURCE_DIR/" "$DOCS_DIR/"

echo "Restoring internal reference docs..."
if [[ -d "$PRESERVE_DIR" ]] && [[ -n "$(ls -A "$PRESERVE_DIR" 2>/dev/null || true)" ]]; then
  rsync -a "$PRESERVE_DIR/" "$DOCS_DIR/"
fi

cat > "$DOCS_DIR/.mintignore" <<'EOF'
# Internal compiler reference (not published on docs.sifr.sh yet).
errors/
schemas/
cli_command_semantics.md
concurrency_runtime.md
formatter.md
linter.md
network_http.md
package_management.md
self_update.md
stdlib_imports.md
text_i18n.md
EOF

cat > "$DOCS_DIR/README.md" <<'EOF'
# Sifr documentation (Mintlify)

Public docs: [docs.sifr.sh](https://docs.sifr.sh)

Source in this directory is deployed by [Mintlify](https://mintlify.com) from `sifr-lang/sifr` (subdirectory `docs`).

## Local preview

```bash
cd docs
npx mint@latest dev
```

## Internal reference (not published)

Compiler reference markdown lives alongside Mintlify pages but is excluded via `.mintignore`:

- `errors/` — generated diagnostic code reference
- `schemas/` — internal schemas
- `*.md` flat files — CLI semantics, formatter, linter, etc.

Migrate these to MDX and add them to `docs.json` when ready to publish.
EOF

echo ""
echo "Import complete. Pages:"
find "$DOCS_DIR" -name '*.mdx' | wc -l | xargs echo "  MDX files:"
echo ""
echo "Next steps:"
echo "  1. cd docs && npx mint@latest validate"
echo "  2. Mintlify dashboard → Git Settings → sifr-lang/sifr, subdirectory docs"
echo "  3. git add docs/ && git commit && git push"
