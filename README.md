# Web Example

```
cargo install trunk --locked
trunk serve
```

# Tauri example

Install the requirements to get started with Tauri

```
cargo install tauri-cli
cargo install trunk --locked
rustup target add wasm32-unknown-unknown
```

Then build the application

```
trunk build
cargo tauri dev
```

## Tauri build requirements

On some systems you may need to install additional dependencies to build Tauri applications. For example, on Fedora you may need:

- libsoup3-devel
- javascriptcoregtk4.1-devel
- webkit2gtk4.1-devel
- maybe more...
