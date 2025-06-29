# Minilink

[![ci_status](https://img.shields.io/github/actions/workflow/status/gibbz00/minilink/ci.yaml?style=for-the-badge)](https://github.com/gibbz00/minilink/actions/workflows/ci.yaml)
[![license](https://img.shields.io/github/license/gibbz00/minilink.svg?style=for-the-badge)](https://github.com/gibbz00/minilink/blob/main/LICENSE.md)
[![crates_io](https://img.shields.io/crates/v/minilink.svg?style=for-the-badge)](https://crates.io/crates/minilink)
[![docs_rs](https://img.shields.io/docsrs/minilink/latest.svg?style=for-the-badge)](https://docs.rs/minilink)

Template and register linker scripts with Rust's conditional compilation flags in hand.

```ld
SECTIONS {
 .text : {
   {% if contains(cfg.feature, "some_feature") %}
   __feature = .;
   {% endif %}
 }
}
```

```rust
minilink::include_template("./ld/link.in.ld", "link.ld");
```

## Further reading

* [API Documentation](https://docs.rs/minilink)
* [Contributing](./CONTRIBUTING.md)
* [Code of conduct](./CODE_OF_CONDUCT.md)
