use crate::interpreter::static_analysis::FunctionInfo;

use super::Analysis;

#[test]
fn hoisted_func_empty() {
    let code = "  \n \n ";

    let analysis = Analysis::analyze(code);

    assert!(analysis.hoisted_funcs.is_empty());
}

#[test]
fn hoisted_func_none() {
    let code = r#"
statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert!(analysis.hoisted_funcs.is_empty());
}

#[test]
fn hoisted_func_none2() {
    let code = r#"
statement!
var var something = 1!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert!(analysis.hoisted_funcs.is_empty());
}

#[test]
fn hoisted_func_minimum() {
    let code = r#"
statement!
statement2!
var var func = =>statement!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_funcs.len(), 1);
    let func = &analysis.hoisted_funcs[0];
    let func_expected = FunctionInfo {
        identifier: "func",
        arg_count: 0,
        hoisted_line: 4,
        body_location: 41,
    };
    assert_eq!(func, &func_expected);
}
