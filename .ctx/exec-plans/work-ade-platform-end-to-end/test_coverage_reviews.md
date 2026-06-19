# Test Coverage Reviews

Record adversarial test coverage reviews and gaps.

## Pending

- Initial coverage review after Work model/CLI slice.
- Final adversarial coverage review before done-ness review.

## Plugin SDK Slice Review

- Added runtime tests for valid examples, ACP provider JSON fixture validation,
  command source qualification, duplicate plugin/provider IDs, collector direct
  store-write rejection, and deferred contribution rejection when embedded in
  the v1 manifest.
- Added adversarial malformed-manifest tests so invalid JSON-like objects return
  diagnostics instead of throwing.
- Added entrypoint field validation coverage for invalid entrypoint kind,
  non-string args, and non-string environment values.
- Added Bazel `unit_tests` and `typecheck` targets and included SDK unit tests
  in `WEB_TESTS`, closing the initial shifted-left coverage gap.
- Remaining gap: hot reload behavior is not covered by this SDK-only slice and
  still requires the plugin registry/reload implementation slice.
