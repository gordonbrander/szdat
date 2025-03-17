## TODO

- [x] Serialize/deserialize CBOR
- [x] Signing / verification
    - [x] Ed25519
- [ ] CLI
  - Rountrip signed envelope + archive

## Development

### Installing binaries on your path with Cargo

From the project directory:

```bash
cargo install --path .
```

This will install the binaries to `~/.cargo/bin`, which is usually added to your path by the Rust installer.
