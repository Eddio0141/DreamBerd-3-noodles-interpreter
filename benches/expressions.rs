use criterion::Criterion;
use dreamberd_3_noodles_interpreter::Interpreter;

pub fn compare_expressions(c: &mut Criterion) {
    c.bench_function("compare_expressions", |b| {
        b.iter(|| {
            Interpreter::new_eval(
                r#"
                print 1 == 2!
                print 1 != 2!
                print 1 < 2!
                print 1 <= 2!
                print 1 > 2!
                print 1 >= 2!
            "#,
            )
            .unwrap();
        });
    });
}
