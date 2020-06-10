# ampmeter-rs.lv2 – a simple amplifier plugin as a prototype

[rust-lv2](https://github.com/RustAudio/rust-lv2) are Rust bindings to write
[LV2 Plugins](https://lv2plug.in). The crate in the repo
[lv2-ui](https://github.com/johannes-mueller/lv2-ui) aims to fill the gap in
rust-lv2 for LV2 GUIs. This repo is the DSP part of a LV2 plugin written in
Rust serving as a test plugin for LV2 UI in Rust.

## Build and installation

Run `cargo build` or `cargo build --release` and the build process should run through.

Then symlink `lv2-debug` or `lv2-release` to `$HOME/.lv2/ampmeter-rs`.

If everything worked out, you can use a plugin host like carla or jalv to run
the plugin `https://johannes-mueller.org/lv2/ampmeter-rs#lv2`.
