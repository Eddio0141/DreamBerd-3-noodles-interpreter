use criterion::{black_box, Criterion};
use dreamberd_noodles_interpreter::Interpreter;

pub fn compare_expressions(c: &mut Criterion) {
    let interpreter = Interpreter::new();

    c.bench_function("compare_expressions", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
                    r#"
var var a = 1 == 2!
a = 1 ;= 2!
a = 1 < 2!
a = 1 <= 2!
a = 1 > 2!
a = 1 >= 2!
            "#,
                ))
                .unwrap();
        });
    });
}

pub fn compare_expressions_chain(c: &mut Criterion) {
    let interpreter = Interpreter::new();

    c.bench_function("compare_expressions_chain", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
                    r#"
var var a = 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1 == 1!
a = 1==1 == 1  ==  1   ==   1    ==    1     ==     1      ==      1       ==       1        ==        1         ==         1          ==          1!
a = 1          ==          1         ==         1        ==        1       ==       1      ==      1     ==     1    ==    1   ==   1  ==  1 == 1!
            "#,
                ))
                .unwrap();
        });
    });
}

pub fn unary_chain(c: &mut Criterion) {
    let interpreter = Interpreter::new();

    c.bench_function("unary_chain", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
                    r#"
var var a = ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;true!
a = ----------------------------------------------------------------------------------------------------1!
            "#,
                ))
                .unwrap();
        });
    });
}

pub fn unary_chain_spaced(c: &mut Criterion) {
    let interpreter = Interpreter::new();

    c.bench_function("unary_chain_spaced", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
                    r#"
var var a = ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; ; true!
a = - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - 1!
            "#,
                ))
                .unwrap();
        });
    });
}

pub fn obj_initialiser(c: &mut Criterion) {
    let interpreter = Interpreter::new();

    c.bench_function("obj_initialiser", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
                    r#"
var var foo = { 
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

pub fn obj_property_access(c: &mut Criterion) {
    let interpreter = Interpreter::new();
    c.bench_function("obj_property_access", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
                    r#"
var var foo = { 
    a: 1, b: 2, c: 3, d: 4, e: 5, f: 6, g: 7, h: 8, i: 9,
    j: 10, k: 11, l: 12, m: 13, n: 14, o: 15, p: 16, q: 17, r: 18,
    s: 19, t: 20, u: 21, v: 22, w: 23, x: 24, y: 25, z: 26,
    __proto__: { __proto__: { __proto__: { __proto__: { __proto__: { inner: 27 } } } } }
}!

assert foo.a === 1!
assert foo.b === 2!
assert foo.c === 3!
assert foo.d === 4!
assert foo.e === 5!
assert foo.inner === 27!
            "#,
                ))
                .unwrap();
        });
    });
}
