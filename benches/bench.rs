use criterion::{criterion_group, criterion_main};

mod expressions;
mod function;
mod variable;

criterion_group!(
    benches,
    variable::declare_many,
    expressions::compare_expressions,
    expressions::compare_expressions_chain,
    expressions::unary_chain,
    expressions::unary_chain_spaced,
    expressions::obj_initialiser,
    expressions::obj_property_access,
    function::many_args
);
criterion_main!(benches);
