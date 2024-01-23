use criterion::{black_box, Criterion};
use dreamberd_noodles_interpreter::InterpreterBuilder;

pub fn compare_expressions(c: &mut Criterion) {
    let mut stdout = std::io::sink();
    let interpreter = InterpreterBuilder::with_stdout(&mut stdout).build();

    c.bench_function("compare_expressions", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
                    r#"
print 1 == 2!
print 1 ;= 2!
print 1 < 2!
print 1 <= 2!
print 1 > 2!
print 1 >= 2!
            "#,
                ))
                .unwrap();
        });
    });
}

pub fn compare_expressions_chain(c: &mut Criterion) {
    let mut stdout = std::io::sink();
    let interpreter = InterpreterBuilder::with_stdout(&mut stdout).build();

    c.bench_function("compare_expressions_chain", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
                    r#"
print 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1!
print 1==1 == 1  ==  1   ==   1    ==    1     ==     1      ==      1       ==       1        ==        1         ==         1          ==          1!
print 1          ==          1         ==         1        ==        1       ==       1      ==      1     ==     1    ==    1   ==   1  ==  1 == 1!
            "#,
                ))
                .unwrap();
        });
    });
}

pub fn unary_chain(c: &mut Criterion) {
    let mut stdout = std::io::sink();
    let interpreter = InterpreterBuilder::with_stdout(&mut stdout).build();

    c.bench_function("unary_chain", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
                    r#"
print ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;true!
print ----------------------------------------------------------------------------------------------------1!
            "#,
                ))
                .unwrap();
        });
    });
}

pub fn unary_chain_spaced(c: &mut Criterion) {
    let mut stdout = std::io::sink();
    let interpreter = InterpreterBuilder::with_stdout(&mut stdout).build();

    c.bench_function("unary_chain_spaced", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
                    r#"
print ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; true!
print - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - 1!
            "#,
                ))
                .unwrap();
        });
    });
}

pub fn obj_initialiser(c: &mut Criterion) {
    let mut stdout = std::io::sink();
    let interpreter = InterpreterBuilder::with_stdout(&mut stdout).build();

    c.bench_function("obj_initialiser", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
                    r#"
var foo = { 
    a: 1, b: 2, c: 3, d: 4, e: 5, f: 6, g: 7, h: 8, i: 9,
    j: 10, k: 11, l: 12, m: 13, n: 14, o: 15, p: 16, q: 17, r: 18,
    s: 19, t: 20, u: 21, v: 22, w: 23, x: 24, y: 25, z: 26
}!
            "#,
                ))
                .unwrap();
        });
    });
}
