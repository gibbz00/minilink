#![allow(missing_docs)]

fn main() {
    // For if statement in `test.in.ld`
    unsafe { std::env::set_var("CARGO_CFG_TEST", "true") };

    minilink::register_template("./ld/test.in.ld", "test.ld");
}
