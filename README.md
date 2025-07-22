# Open Mediatek Generic HAL
Collection of open-source Mediatek HALs written in Rust. Most library crates support `#![no_std]`!

## Building
For cross-compilation it's recommended to use `cargo zigbuild`:
```shell
$ cargo zigbuild --target aarch64-unknown-linux-musl --release
```

Building `std` from source can greatly benefit the binary size:
```shell
$ cargo zigbuild -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target aarch64-unknown-linux-musl --release
```
