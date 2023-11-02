use criterion::{criterion_group, criterion_main};

mod expressions;
mod variable;

criterion_group!(
    benches,
    variable::declare_many,
    expressions::compare_expressions,
    expressions::compare_expressions_chain,
    expressions::unary_chain,
    expressions::unary_chain_spaced
);
criterion_main!(benches);
