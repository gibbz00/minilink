//! # Minilink - In search of better linker script composition
//!
//! Template and register linker scripts with Rust's conditional compilation flags in hand.
//!
//! More specifically, use `minijinja` for templating within build scripts. Linker scripts may
//! templated and added to the linker script search path with [`register_template`]. Scripts may
//! alternatively be added directly by using [`include_template`], bypassing the need to manually
//! configure `-T` link arguments.
//!
//! Note that these function should only be called within `build.rs` scripts. Not only are errors
//! handled with `expect`, but `println!` statements are used for emitting cargo build instructions.
//!
//! ## Minijinja Environment
//!
//! The minijinja render environment contains the following context entries:
//!
//! - `cfg`: Map of all registered configuration options for the package being built. Names are
//!   lower cased. Values may be lists or singular strings. `true` cfg features are represented as
//!   empty strings (`""`) since Cargo does not create `CARGO_CFG_<cfg>` environment variables for
//!   boolean features whose values are `false`.
//!
//! The following custom functions also registered:
//!
//! - `contains(<cfg_key>, <value>)`: Covers the case when a map value may be either a singular
//!   string or a list of strings. For example; `contains(cfg.feature, "alloc")` works when there is
//!   only one feature (i.e. `cfg = "alloc"`), or multiple (e.g. `cfg = ["alloc", "std"]`).
//!
//! # Example
//!
//! The a templated linker script in `./ld/link.in.ld`:
//!
//! ```ld
//! SECTIONS {
//!  .text : {
//!    {% if contains(cfg.feature, "some_feature") %}
//!    __feature = .;
//!    {% endif %}
//!  }
//! }
//! ```
//!
//! Can be registered in a `build.rs` with:
//!
//! ```ignore
//! minilink::register_template("./ld/link.in.ld", "link.ld");
//! ```
//!
//! Which in turn produces a `$OUT_DIR/link.ld` containing:
//!
//! ```ld
//! SECTIONS {
//!  .text : {
//!    __feature = .;
//!  }
//! }
//! ```

use std::error::Error;

mod context;
pub(crate) use context::{LinkerTemplateContext, TemplateContextCfg};

/// Register a templated linker script
///
/// See crate level documentation for an introduction.
///
/// Unless an absolute template input file path is provided, it will be relative to the cargo
/// package manifest (e.g Cargo.toml).
///
/// Note that the output linker script must be explicitly included by the final since it is only
/// added to the linker script search path. This allows application developers to control the
/// order in which linker scripts are combined. The recommended way of doing so would be to create a
/// `linkall.ld` script which uses the `INCLUDE` command to pull in the scrips in the
/// desired order. `linkall.ld` could in turn be passed to [`include_template`].
pub fn register_template(path_in: impl AsRef<std::path::Path>, name_out: &str) {
    template_impl(path_in.as_ref(), name_out, false);
}

/// Like [`register_template`], but the templates are included immediately
///
/// This removes the need to exclicitly include it in downstream crates, but
/// also the ability control the linker script order.
pub fn include_template(path_in: impl AsRef<std::path::Path>, name_out: &str) {
    template_impl(path_in.as_ref(), name_out, true);
}

fn template_impl(path_in: &std::path::Path, name_out: &str, add_immediately: bool) {
    println!("cargo::rerun-if-changed={}", path_in.display());

    let linker_script_in = std::fs::read_to_string(path_in).expect("failed to read input linker script");

    let linker_script_out = process(&linker_script_in).expect("failed to process linker script template");

    let out_dir = std::env::var("OUT_DIR").expect("target output directory not found");
    println!("cargo::rustc-link-search={out_dir}");

    if add_immediately {
        // Linkers treat an unknown format as a linker script. Doing this over
        // rustc-link-arg=-T allows the script to also be passed to dependant
        // crates, opposed to only the crate being built.
        println!("cargo::rustc-link-lib=dylib:+verbatim={name_out}");
    }

    let path_out = std::path::Path::new(&out_dir).join(name_out);

    std::fs::write(path_out, linker_script_out)
        .expect("unable to write process linker script to target output directory");
}

fn process(linker_script_in: &str) -> Result<String, Box<dyn Error>> {
    let mut env = minijinja::Environment::new();
    // Indentation
    env.set_lstrip_blocks(true);
    env.set_trim_blocks(true);
    // Custom functions
    env.add_function("contains", custom_functions::contains);

    env.template_from_str(linker_script_in)?
        .render(LinkerTemplateContext::new())
        .map_err(Into::into)
}

mod custom_functions {
    use minijinja::value::ViaDeserialize;

    use crate::*;

    pub(crate) fn contains(maybe_cfg: ViaDeserialize<Option<TemplateContextCfg>>, value: String) -> bool {
        maybe_cfg.as_ref().is_some_and(|cfg| match cfg {
            TemplateContextCfg::Value(existing_value) => existing_value == &value,
            TemplateContextCfg::List(items) => items.contains(&value),
        })
    }
}
