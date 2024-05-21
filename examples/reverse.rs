use dreamberd_noodles_interpreter::Interpreter;

fn main() {
    Interpreter::new_eval(include_str!("reverse.db")).unwrap();
}
