//! # Minilink
//!
//! Template linker scripts with a conditional compilation context.

use std::{collections::HashMap, error::Error};

/// Register a `minijinja` templated linker script
///
/// The script is only added to the link search path, which means that it must
/// be included by the build script of the final application. This allows the
/// application developers to control the order in which linker scripts are
/// combined. The recommended way of doing so would be to create a `linkall.ld`
/// script which uses the `INCLUDE` command to pull in the scrips in the
/// desired order. This script is then passed to [`include_template`].
///
/// Note that the script should only be called in `build.rs` scripts. Not only
/// are errors handled with `expect`, but `println!` statements are called for
/// emitting cargo build instructions.
///
/// Adds `rerun-if-changed` to the input template, and an `rustc-link-search` for the output
/// location of the processed linker script. The `name_out` parameter is used to create the
/// output file name `<crate-name>_<name_out>` placed in `$OUT_DIR`. `<crate_name>` is retrieved
/// through the `CARGO_PKG_NAME` environment variable.
///
/// ## Minijinja context
///
/// Contains the following entries:
///
/// - `cfg`: Map of all registered configuration options for the package being built. Names are
///   lower cased. Values may be lists or singular strings. `true` cfg features are represented as
///   empty strings (""). (Cargo does not create `CARGO_CFG_<cfg>` environment variables for boolean
///   features whose values are `false`.)
///
/// And custom functions:
///
/// - `contains(<cfg_key>, <value>)`: ex. `contains(cfg.feature, "alloc")`
pub fn register_template(path_in: impl AsRef<std::path::Path>, name_out: &str) {
    template_impl(path_in.as_ref(), name_out, false, true);
}

/// Like [`register_template`] but without the crate name prefixed to the output linker script name
pub fn register_template_without_prefix(path_in: impl AsRef<std::path::Path>, name_out: &str) {
    template_impl(path_in.as_ref(), name_out, true, false);
}

/// Like [`register_template`], but the templates are included immediatedly
///
/// This removes the need to exclicitly include it in downstream crates, but
/// also the ability control the linker script order.
pub fn include_template(path_in: impl AsRef<std::path::Path>, name_out: &str) {
    template_impl(path_in.as_ref(), name_out, true, true);
}

fn template_impl(path_in: &std::path::Path, name_out: &str, add_immediatedly: bool, prefix_crate_name: bool) {
    println!("cargo::rerun-if-changed={}", path_in.display());

    let linker_script_in = std::fs::read_to_string(path_in).expect("failed to read input linker script");

    let linker_script_out = process(&linker_script_in).expect("failed to process linker script template");

    let out_dir = std::env::var("OUT_DIR").expect("target output directory not found");
    println!("cargo::rustc-link-search={out_dir}");

    let linker_script_name = match prefix_crate_name {
        true => {
            let crate_name = std::env::var("CARGO_PKG_NAME").expect("failed to retrieve cargo package name");
            format!("{crate_name}_{name_out}")
        }
        false => name_out.to_owned(),
    };

    if add_immediatedly {
        // linkers treat an unknown format as a linker script, doing this
        // over rustc-link-arg=-T allows the script inclusion to dependant
        // crates
        println!("cargo::rustc-link-lib=dylib:+verbatim={linker_script_name}");
    }

    let path_out = std::path::Path::new(&out_dir).join(linker_script_name);

    std::fs::write(path_out, linker_script_out)
        .expect("unable to write process linker script to target output directory");
}

pub(crate) fn process(linker_script_in: &str) -> Result<String, Box<dyn Error>> {
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

    pub fn contains(maybe_cfg: ViaDeserialize<Option<TemplateContextCfg>>, value: String) -> bool {
        maybe_cfg.as_ref().is_some_and(|cfg| match cfg {
            TemplateContextCfg::Value(existing_value) => existing_value == &value,
            TemplateContextCfg::List(items) => items.contains(&value),
        })
    }
}

#[derive(serde::Serialize)]
struct LinkerTemplateContext {
    cfg: HashMap<String, TemplateContextCfg>,
}

impl LinkerTemplateContext {
    fn new() -> Self {
        let cfg = std::env::vars()
            .filter_map(|(name, value)| {
                name.strip_prefix("CARGO_CFG_")
                    .map(|name| (name.to_lowercase(), TemplateContextCfg::from_env_value(value)))
            })
            .collect();

        Self { cfg }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum TemplateContextCfg {
    Value(String),
    List(Vec<String>),
}

impl TemplateContextCfg {
    fn from_env_value(str_value: String) -> Self {
        if str_value.contains(',') {
            let values = str_value.split(',').map(ToOwned::to_owned).collect();
            TemplateContextCfg::List(values)
        } else {
            TemplateContextCfg::Value(str_value)
        }
    }
}
