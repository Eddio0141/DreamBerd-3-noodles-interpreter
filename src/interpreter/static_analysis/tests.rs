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

#[test]
fn hoisted_func_minimum_wide() {
    let code = r#"
statement!
var var func =   =>   statement   !
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_funcs.len(), 1);
    let func = &analysis.hoisted_funcs[0];
    let func_expected = FunctionInfo {
        identifier: "func",
        arg_count: 0,
        hoisted_line: 3,
        body_location: 34,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_arg_minimum() {
    let code = r#"
statement!
var var func = arg =>statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_funcs.len(), 1);
    let func = &analysis.hoisted_funcs[0];
    let func_expected = FunctionInfo {
        identifier: "func",
        arg_count: 1,
        hoisted_line: 3,
        body_location: 33,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_arg_wide() {
    let code = r#"
statement!
var var func = arg   =>   statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_funcs.len(), 1);
    let func = &analysis.hoisted_funcs[0];
    let func_expected = FunctionInfo {
        identifier: "func",
        arg_count: 1,
        hoisted_line: 3,
        body_location: 38,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_args_minimum() {
    let code = r#"
statement!
var var func = arg,arg2,arg3 =>   statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_funcs.len(), 1);
    let func = &analysis.hoisted_funcs[0];
    let func_expected = FunctionInfo {
        identifier: "func",
        arg_count: 3,
        hoisted_line: 3,
        body_location: 46,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_args_wide() {
    let code = r#"
statement!
var var func = arg , arg2, arg3, arg4 =>   statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_funcs.len(), 1);
    let func = &analysis.hoisted_funcs[0];
    let func_expected = FunctionInfo {
        identifier: "func",
        arg_count: 4,
        hoisted_line: 3,
        body_location: 55,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_args_weird() {
    let code = r#"
statement!
var var func = arg, => , => => statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_funcs.len(), 1);
    let func = &analysis.hoisted_funcs[0];
    let func_expected = FunctionInfo {
        identifier: "func",
        arg_count: 3,
        hoisted_line: 3,
        body_location: 43,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_life_time_line_neg() {
    let code = r#"
statement!
var var func<-2> = arg, arg2 => statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_funcs.len(), 1);
    let func = &analysis.hoisted_funcs[0];
    let func_expected = FunctionInfo {
        identifier: "func",
        arg_count: 2,
        hoisted_line: 1,
        body_location: 44,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_life_time_line_pos() {
    let code = r#"
statement!
var var func<2> = arg, arg2 => statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_funcs.len(), 1);
    let func = &analysis.hoisted_funcs[0];
    let func_expected = FunctionInfo {
        identifier: "func",
        arg_count: 2,
        hoisted_line: 3,
        body_location: 43,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_life_time_seconds() {
    let code = r#"
statement!
var var func<50s> = arg, arg2 => statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_funcs.len(), 1);
    let func = &analysis.hoisted_funcs[0];
    let func_expected = FunctionInfo {
        identifier: "func",
        arg_count: 2,
        hoisted_line: 3,
        body_location: 45,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_life_time_infinity() {
    let code = r#"
statement!
var var func<Infinity> = arg, arg2 => statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_funcs.len(), 1);
    let func = &analysis.hoisted_funcs[0];
    let func_expected = FunctionInfo {
        identifier: "func",
        arg_count: 2,
        hoisted_line: 3,
        body_location: 50,
    };
    assert_eq!(func, &func_expected);
}
