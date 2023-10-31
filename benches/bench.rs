use criterion::{criterion_group, criterion_main};

mod expressions;
mod variable;

criterion_group!(
    benches,
    variable::declare_many,
    expressions::compare_expressions
);
criterion_main!(benches);
