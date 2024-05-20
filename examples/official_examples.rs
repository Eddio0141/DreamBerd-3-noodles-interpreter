use dreamberd_noodles_interpreter::Interpreter;

fn main() {
    // TODO: fix official examples to actually match real ones
    Interpreter::new_eval(include_str!("fizzbuzz.db")).unwrap();
}
