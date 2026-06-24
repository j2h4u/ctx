# Production Readiness

Production readiness for ctx means the local search product is understandable,
bounded, and testable.

## Product Readiness

- Public docs describe only local provider import, search, and context.
- A fresh agent can initialize storage, import supported local history, search,
  inspect results, and build cited context from the docs.
- Provider support is truthful and machine-readable.
- Limitations are documented next to the happy path.

## Security Readiness

- Command read/write behavior is documented.
- Core setup/import/search/context are local operations.
- JSON output is private by default.
- Raw provider ownership is clear.
- Redaction limits are explicit.

## Contract Readiness

- CLI examples match implemented flags.
- JSON fields used by agents are documented.
- Remaining compatibility names are treated as opaque implementation details.
- Source citation and source availability semantics are documented.

## Validation Readiness

The preferred validation gate is:

```bash
bazel test //:docs_check --config=ci
```

Until that target exists, use static docs checks:

```bash
bash scripts/check-docs.sh
jq empty docs/provider-support-matrix.json
```

Do not use direct Cargo checks for docs-only validation unless the execution
plan is updated to require them through Bazel.
