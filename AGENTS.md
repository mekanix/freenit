# Project conventions

- Throughout the project, every module with its own directory must use `<directory>/mod.rs` as its module root. Do not create a sibling `<directory>.rs` file alongside `<directory>/`.
- For example, use `src/app/mail/mod.rs`, not `src/app/mail.rs`. This applies to `app`, `chota`, `features`, `db`, and every other module tree.
- Strongly prefer pure Rust over JavaScript. Use Topcoat signals, runtime expressions, procedures, and CSS for browser behavior whenever they can provide the required functionality. Do not add handwritten JavaScript for behavior that can be implemented in Rust; reserve JavaScript for genuine requirements such as generated `wasm-bindgen` glue.

## Required checks

After implementing a change, run both commands:

```sh
cargo clippy --all-targets --fix --allow-dirty --all-features
topcoat fmt
```
