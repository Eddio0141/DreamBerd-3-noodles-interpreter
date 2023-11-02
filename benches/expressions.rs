use criterion::{black_box, Criterion};
use dreamberd_3_noodles_interpreter::interpreter::InterpreterBuilder;

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
