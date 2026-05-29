# eda4 Web

Web interface for eda4 tools — Actix-web backend + static HTML/JS frontend.

## Tabs

| Tab | Feature |
|---|---|
| Verilog Sim | Run Verilog simulation via `verilog2rust` |
| Verilog PnR | Run Verilog → PnR pipeline via `v2f-*` |
| SPICE | Run SPICE circuit simulation via `ruspice` |
| Bitstream | Decode iCE40 bitstream via `v2f-bitdecode` |

## Quick start

```bash
# Start server
cargo run -p eda4-web-server

# Open in browser
open http://localhost:8080
```

## Test

```bash
# Backend tests
cargo test -p eda4-web-server

# E2E (Playwright) — needs server running on :8080
cd frontend && npx playwright test
```

## Scripts

```bash
./web.sh    # Kill old server, start fresh, run Playwright e2e
```

## Structure

```
web/
├── server/         Actix-web backend (Rust)
│   ├── Cargo.toml
│   └── src/
├── frontend/       Static HTML + JS
│   ├── index.html
│   ├── js/
│   ├── tests/      Playwright e2e tests
│   └── playwright.config.js
└── web.sh
```
