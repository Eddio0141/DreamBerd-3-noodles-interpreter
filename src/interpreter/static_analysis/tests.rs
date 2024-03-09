use crate::interpreter::static_analysis::HoistedVarInfo;

use super::Analysis;

#[test]
fn hoisted_func_empty() {
    let code = "  \n \n ";

    let analysis = Analysis::analyze(code);

    assert!(analysis.hoisted_vars.is_empty());
}

#[test]
fn hoisted_func_none() {
    let code = r#"
statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert!(analysis.hoisted_vars.is_empty());
}

#[test]
fn hoisted_func_none2() {
    let code = r#"
statement!
var var something = 1!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert!(analysis.hoisted_vars.is_empty());
}

#[test]
fn hoisted_func_minimum() {
    let code = r#"
statement!
statement2!
var var func<-2> = =>statement!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let func = &analysis.hoisted_vars[0];
    let func_expected = HoistedVarInfo {
        identifier: "func".to_string(),
        hoisted_line: 2,
        expr_index: 43,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_minimum_wide() {
    let code = r#"
statement!
var var func<-1> =   =>   statement   !
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let func = &analysis.hoisted_vars[0];
    let func_expected = HoistedVarInfo {
        identifier: "func".to_string(),
        hoisted_line: 2,
        expr_index: 33,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_arg_minimum() {
    let code = r#"
statement!
var var func<-1> = arg =>statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let func = &analysis.hoisted_vars[0];
    let func_expected = HoistedVarInfo {
        identifier: "func".to_string(),
        hoisted_line: 2,
        expr_index: 31,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_arg_wide() {
    let code = r#"
statement!
var var func<-1> = arg   =>   statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let func = &analysis.hoisted_vars[0];
    let func_expected = HoistedVarInfo {
        identifier: "func".to_string(),
        hoisted_line: 2,
        expr_index: 31,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_args_minimum() {
    let code = r#"
statement!
var var func<-1> = arg,arg2,arg3 =>   statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let func = &analysis.hoisted_vars[0];
    let func_expected = HoistedVarInfo {
        identifier: "func".to_string(),
        hoisted_line: 2,
        expr_index: 31,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_args_wide() {
    let code = r#"
statement!
var var func<-1> = arg , arg2, arg3, arg4 =>   statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let func = &analysis.hoisted_vars[0];
    let func_expected = HoistedVarInfo {
        identifier: "func".to_string(),
        hoisted_line: 2,
        expr_index: 31,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn hoisted_func_args_weird() {
    let code = r#"
statement!
var var func<-1> = arg, => , => => statement!
statement2!
"#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let func = &analysis.hoisted_vars[0];
    let func_expected = HoistedVarInfo {
        identifier: "func".to_string(),
        hoisted_line: 2,
        expr_index: 31,
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

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let func = &analysis.hoisted_vars[0];
    let func_expected = HoistedVarInfo {
        identifier: "func".to_string(),
        hoisted_line: 1,
        expr_index: 31,
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

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let func = &analysis.hoisted_vars[0];
    let func_expected = HoistedVarInfo {
        identifier: "func".to_string(),
        hoisted_line: 3,
        expr_index: 30,
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

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let func = &analysis.hoisted_vars[0];
    let func_expected = HoistedVarInfo {
        identifier: "func".to_string(),
        hoisted_line: 3,
        expr_index: 32,
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

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let func = &analysis.hoisted_vars[0];
    let func_expected = HoistedVarInfo {
        identifier: "func".to_string(),
        hoisted_line: 3,
        expr_index: 37,
    };
    assert_eq!(func, &func_expected);
}

#[test]
fn const_const() {
    let code = r#"
print foo!
const const value<-1> = 5!
    "#;

    let analysis = Analysis::analyze(code);

    assert_eq!(analysis.hoisted_vars.len(), 1);
    let var = &analysis.hoisted_vars[0];
    let var_expected = HoistedVarInfo {
        identifier: "value".to_string(),
        hoisted_line: 2,
        expr_index: 36,
    };
    assert_eq!(var, &var_expected);
}
