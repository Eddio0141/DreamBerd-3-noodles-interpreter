use criterion::{black_box, Criterion};
use dreamberd_noodles_interpreter::InterpreterBuilder;

pub fn declare_many(c: &mut Criterion) {
    let mut stdout = std::io::sink();
    let interpreter = InterpreterBuilder::with_stdout(&mut stdout).build();

    c.bench_function("declare_many", |b| {
        b.iter(|| {
            interpreter
                .eval(black_box(
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
