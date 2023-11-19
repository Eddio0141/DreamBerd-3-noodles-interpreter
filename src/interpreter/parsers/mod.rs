use std::borrow::Borrow;

use nom::{branch::*, bytes::complete::*, combinator::*, error::*, multi::*, sequence::*, *};

use self::types::*;

pub mod types;

pub static WHITESPACE: [char; 6] = [' ', '\t', '\n', '\r', '(', ')'];

pub fn is_ws(ch: char) -> bool {
    WHITESPACE.contains(&ch)
}

/// At least one whitespace repeated
pub fn ws1(input: Position) -> PosResult<()> {
    take_while1(is_ws).map(|_| ()).parse(input)
}

/// Any amount of whitespace repeated
pub fn ws(input: Position) -> PosResult<()> {
    take_while(is_ws).map(|_| ()).parse(input)
}

pub fn identifier<'a, I, E, P, PO>(terminating_parser: P) -> impl Fn(I) -> IResult<I, I, E>
where
    I: InputTakeAtPosition<Item = char> + InputIter + InputTake + Borrow<str> + Copy + InputLength,
    E: ParseError<I>,
    P: Parser<I, PO, E> + Copy,
{
    move |input| {
        let ws_char = || {
            verify(take(1usize), |s: &str| {
                !s.is_empty() && !is_ws(s.chars().next().unwrap())
            })
        };
        let identifier = tuple((
            ws_char(),
            many0_count(not(alt((
                ws_char().map(|_| ()),
                terminating_parser.map(|_| ()),
            )))),
        ));
        let (_, (_, rest)) = peek(identifier)(input)?;

        // rest + 1 character
        Ok(input.take_split(rest + 1))
    }
}
