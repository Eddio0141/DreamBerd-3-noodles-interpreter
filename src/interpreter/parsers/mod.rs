use nom::{bytes::complete::*, *};

use self::types::*;

pub mod types;

pub static WHITESPACE: [char; 6] = [' ', '\t', '\n', '\r', '(', ')'];

pub fn is_ws(ch: char) -> bool {
    WHITESPACE.contains(&ch)
}

/// At least one whitespace repeated
pub fn ws1<'a, Input>(input: Position) -> PosResult<()>
where
    Input: InputTakeAtPosition<Item = char>,
{
    take_while1(is_ws).map(|_| ()).parse(input)
}

/// Any amount of whitespace repeated
pub fn ws<'a, Input>(input: Input) -> IResult<Input, Input>
where
    Input: InputTakeAtPosition<Item = char>,
{
    take_while(is_ws)(input)
}
