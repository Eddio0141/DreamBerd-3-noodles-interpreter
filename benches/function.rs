use criterion::{black_box, Criterion};
use dreamberd_noodles_interpreter::Interpreter;

pub fn many_args(c: &mut Criterion) {
    c.bench_function("many_args", |b| {
        b.iter(|| {
            Interpreter::new_eval(black_box(
                r#"
function some_calculation(a, b, c, d, e, f, g) => a + b + c + d + e + f + g!
some_calculation(1, 2, 3, 4, 5, 6, 7)!
        "#,
            ))
        })
    });
}
