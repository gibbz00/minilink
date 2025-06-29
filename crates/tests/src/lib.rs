#![allow(missing_docs)]
#![no_std]

#[cfg(test)]
mod tests {
    const EXPECTED: &str = r#"SECTIONS {
	.test : {
		__test = .;
	}
}"#;

    #[test]
    fn templates_linker_script() {
        let linker_script = include_str!(concat!(env!("OUT_DIR"), "/test.ld"));
        assert_eq!(EXPECTED, linker_script)
    }
}
