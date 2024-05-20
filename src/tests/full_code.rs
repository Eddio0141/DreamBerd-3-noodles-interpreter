use crate::Interpreter;

// TODO: add rest

#[test]
// TODO: use original
// TODO: assert output
fn fizz_buzz() {
    let code = r#"
when i % 3 === 0  &&  i % 5 === 0 {
    print "FizzBuzz"!
} else when i % 3 === 0 {
    print "Fizz"!
} else when i % 5 === 0 {
    print "Buzz"!
} else {
    print i!
}

when i < 20 {
    i = i + 1!
}

i = 0!
"#;
    Interpreter::new_eval(code).unwrap();
}
