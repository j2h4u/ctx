# Work Recorder Dashboard

Local React/Vite dashboard used by `ctx dashboard export`.

- React renders a static, local-only SPA from `#ctx-dashboard-data`.
- Rust owns the share-safe normalized DTO and embeds it into `dist/index.html`.
- The UI uses Tailwind styles, Radix tabs, TanStack Table, and Recharts without importing ADE runtime state.
- Playwright screenshots are written to `target/ctx-artifacts/dashboard-react`.

Useful commands:

```bash
npm install
npm run build
npm run test
```
