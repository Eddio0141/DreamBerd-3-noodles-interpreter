use crate::Interpreter;

#[test]
fn if_statement_true() {
    let code = r#"
var var x = 1!
if x === 1 {
    x = 2!
}
assert x === 2!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn if_statement_false() {
    let code = r#"
var var x = 1!
if x === 2 {
    x = 3!
}
assert x === 1!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn if_else_statement_true() {
    let code = r#"
var var x = 1!
if x === 1 {
    x = 2!
} else {
    x = 3!
}
assert x === 2!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn if_else_statement_false() {
    let code = r#"
var var x = 2!
if x === 1 {
    x = 2!
} else {
    x = 3!
}
assert x === 3!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn if_elseif_statement_true() {
    let code = r#"
var var x = 1!
if x === 1 {
    x = 2!
} else if x === 2 {
    x = 3!
}
assert x === 2!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn if_elseif_statement_true2() {
    let code = r#"
var var x = 2!
if x === 1 {
    x = 2!
} else if x === 2 {
    x = 3!
}
assert x === 3!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn if_elseif_else_statement_true() {
    let code = r#"
var var x = 1!
if x === 1 {
    x = 2!
} else if x === 2 {
    x = 3!
} else {
    x = 4!
}
assert x === 2!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn if_elseif_else_statement_true2() {
    let code = r#"
var var x = 2!
if x === 1 {
    x = 2!
} else if x === 2 {
    x = 3!
} else {
    x = 4!
}
assert x === 3!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn if_elseif_else_statement_false() {
    let code = r#"
var var x = 3!
if x === 1 {
    x = 2!
} else if x === 2 {
    x = 3!
} else {
    x = 4!
}
assert x === 4!
"#;
    Interpreter::new_eval(code).unwrap();
}
