# Blog Project Workspace

This repository is organized as a Cargo workspace with multiple crates:

- blog-server — Web server for the blog (binary)
- blog-client — Client library (library)
- blog-cli — CLI client (binary) depending on a blog-client
- blog-wasm — WASM frontend (library)

## Getting started

- Build all crates:
  - `cargo build --workspace`
- Run the server:
  - `cargo run -p blog-server`
- Run the CLI:
  - `cargo run -p blog-cli -- --help`

## Layout

blog-project/
├── Cargo.toml
├── README.md
├── blog-server/
├── blog-client/
├── blog-cli/
└── blog-wasm/

