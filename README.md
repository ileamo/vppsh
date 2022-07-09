1. Install rust, see https://www.rust-lang.org/tools/install
2. Optionaly install additional packages and compile i18n files
```bash
cargo install cargo-i18n
cargo install xtr
cargo i18n
```
3. Run vppsh
```bash
cargo run
```

You must have vpp running and properly permissions.
```bash
sudo usermod -aG vpp <USER>
```

To build release exec
```bash
cargo build --release
./target/release/vppsh
```
