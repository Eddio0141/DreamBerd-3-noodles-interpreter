use criterion::{black_box, Criterion};
use dreamberd_3_noodles_interpreter::Interpreter;

pub fn declare_many(c: &mut Criterion) {
    c.bench_function("declare_many", |b| {
        b.iter(|| {
            Interpreter::new_eval(black_box(
                r#"
    var var a = 0!
    var var b = 0!
    var var c = 0!
    var var d = 0!
    var var e = 0!
    var var f = 0!
    var var g = 0!
    var var h = 0!
    var var i = 0!
    var var j = 0!
    var var k = 0!
    var var l = 0!
    var var m = 0!
    var var n = 0!
    var var o = 0!
        "#,
            ))
            .unwrap()
        })
    });
}
