The `vppsh` is a wrapper around `vppctl` for more convenient VPP configuration and monitoring.
> **_NOTE:_** The vppsh is in alpha stage
## Quick start
* Install Rust, see https://www.rust-lang.org/tools/install
* Run vppsh
```bash
cargo run
```

## Troubleshooting

You must have vpp running and the appropriate permissions
```bash
sudo usermod -aG vpp <USER>
```

## Build a release
```bash
cargo build --release
./target/release/vppsh
```

## Extract, build and verify localization resources
Install additional packages and run `i18n` command
```bash
cargo install cargo-i18n
cargo install xtr
cargo i18n
```
You have to run `cargo i18n` after every `tr!()` macro editing or adding new translation in *.po files.