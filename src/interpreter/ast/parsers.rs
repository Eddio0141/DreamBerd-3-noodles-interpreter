use crate::interpreter::parsers::*;
use nom::{
    bytes::complete::{tag, take_while, take_while1},
    error::ParseError,
    sequence::Tuple,
};

/// At least one whitespace repeated
fn ws1<'a, Error>() -> impl Fn(&'a str) -> nom::IResult<&'a str, &'a str, Error>
where
    Error: ParseError<&'a str>,
{
    take_while1(|c| WHITESPACE.contains(&c))
}

/// Any amount of whitespace repeated
pub fn ws<'a, Error>() -> impl Fn(&'a str) -> nom::IResult<&'a str, &'a str, Error>
where
    Error: ParseError<&'a str>,
{
    take_while(|c| WHITESPACE.contains(&c))
}

pub fn is_var_var(code: &str) -> bool {
    let var = tag::<_, _, nom::error::Error<_>>("var");
    let ws = ws1();

    let Ok((code, _)) = (var, ws, var).parse(code) else {
        return false;
    };

    todo!()
}
