use super::*;

#[test]
fn function_header() {
    let header = "function";
    assert!(is_function_header(header));
}

#[test]
fn function_header_single_char() {
    let function = "function".chars();

    for c in function {
        assert!(is_function_header(&c.to_string()));
    }
}

#[test]
fn function_header_jumble() {
    let header = "fnto";
    assert!(is_function_header(header));
}

#[test]
fn function_header_invalid() {
    let header: &str = "ffn";
    assert!(!is_function_header(header));
}
