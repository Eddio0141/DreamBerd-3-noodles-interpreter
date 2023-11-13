use crate::interpreter::parsers::*;
use nom::{
    bytes::complete::{tag, take_while, take_while1},
    character::complete,
    error::ParseError,
    multi::many0_count,
    sequence::Tuple,
    IResult,
};

fn is_ws(ch: char) -> bool {
    WHITESPACE.contains(&ch)
}

/// At least one whitespace repeated
pub fn ws1<'a, Error>() -> impl Fn(&'a str) -> IResult<&'a str, &'a str, Error>
where
    Error: ParseError<&'a str>,
{
    take_while1(is_ws)
}

/// Any amount of whitespace repeated
pub fn ws<'a, Error>() -> impl Fn(&'a str) -> IResult<&'a str, &'a str, Error>
where
    Error: ParseError<&'a str>,
{
    take_while(is_ws)
}

pub fn is_var_var(code: &str) -> bool {
    let var = tag::<_, _, nom::error::Error<_>>("var");
    let ws = ws1();

    let Ok((code, _)) = (var, ws, var).parse(code) else {
        return false;
    };

    todo!()
}

pub fn identifier_optional_term<'a, Error>(
    term_char: char,
) -> impl Fn(&'a str) -> IResult<&'a str, &'a str, Error>
where
    Error: ParseError<&'a str>,
{
    // ident -> ws
    // or
    // ident -> term_char -> ws
    let mut got_first_char = false;
    take_while1(|c| {
        let end = is_ws(c) || (got_first_char && term_char == c);
        if end {
            return false;
        }
        if !got_first_char {
            got_first_char = true;
            return true;
        }
        !end
    })
}

pub fn term<'a, Error>() -> impl Fn(&'a str) -> IResult<&'a str, usize, Error>
where
    Error: ParseError<&'a str>,
{
    let term = complete::char('!');
    many0_count(term)
}

pub fn equals<'a, Error>() -> impl Fn(&'a str) -> IResult<&'a str, usize, Error>
where
    Error: ParseError<&'a str>,
{
    complete::char('=')
}
