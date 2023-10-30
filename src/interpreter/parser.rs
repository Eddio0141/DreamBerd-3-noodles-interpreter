use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "interpreter/dreamberd.pest"]
/// Parser for Dreamberd
pub struct PestParser;
