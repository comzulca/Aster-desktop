# Dependency review — 19 November 2026

The review started at main commit `d03189e`. Its [main workflow](https://github.com/comzulca/Aster-desktop/actions/runs/44380841386) passed. The [registry comparison](https://github.com/comzulca/Aster-desktop/actions/runs/44496041826) found six npm updates and Cargo changes in the host and both test harnesses. The [resolution run](https://github.com/comzulca/Aster-desktop/actions/runs/44496399277) used pnpm and Cargo to regenerate the locks.

| Dependency | Previous lock | Updated lock |
| --- | --- | --- |
| @tauri-apps/api and @tauri-apps/cli | 2.12.4 | 2.12.5 |
| @testing-library/user-event | 14.6.8 | 14.6.12 |
| jsdom | 31.0.0 | 31.0.1 |
| Vite | 8.3.6 | 8.3.8 |
| Vitest | 5.1.0 | 5.1.1 |
| tauri (Rust) | 2.12.2 | 2.12.7 |
| base64 (codec fixtures) | 0.22.1 | 0.23.1 |
| sha2 (codec fixtures) | 0.11.0 | 0.11.1 |

The production Cargo graph and the two QA graphs were resolved separately. Cargo retained generic-array 0.14.7, toml 0.8.2, toml_datetime 0.6.3 and toml_edit 0.20.2 in the transitive graph. The Windows ABI guard requires windows-future 0.3.2 alongside windows 0.62.2. The patched zune-jpeg source remains at its reviewed 0.5.15 revision. The checkout, setup-node and upload-artifact actions matched the latest stable upstream releases at the review date.
