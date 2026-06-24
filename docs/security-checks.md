# Security Checks

This page defines the checks public docs and production validation should keep
true for the search-only product.

## Required Invariants

- `ctx setup` creates only the configured ctx data root and local storage files.
- `ctx sources` writes nothing.
- `ctx import` writes only the configured ctx data root and SQLite index.
- `ctx search`, `ctx context`, `ctx list`, and `ctx show` write nothing.
- Core setup/import/search/context do not require network access or API keys.
- Provider files are read as sources and not modified.
- JSON output is private by default and must not be described as share-safe.
- Unsupported providers remain explicit in the provider support matrix.

## Static Docs Checks

Public docs should avoid claims for capabilities outside the product contract.
Run the repository docs check, which scans public copy for removed or
unsupported product surfaces:

```bash
bash scripts/check-docs.sh
```

Validate the provider matrix JSON:

```bash
jq empty docs/provider-support-matrix.json
```

When Bazel owns the docs gate, run:

```bash
bazel test //:docs_check --config=ci
```

## Manual Review Checklist

- README scope matches `docs/product-contract.md`.
- CLI examples use flags implemented by `crates/ctx-cli`.
- Provider support docs match `docs/provider-support-matrix.json`.
- JSON docs identify local/private output and compatibility limits.
- Security docs do not promise sanitization beyond bounded previews and
  share-safety markers.
- Release install docs do not imply public artifacts before they exist.
