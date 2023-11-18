use nom::{bytes::complete::*, *};

pub static WHITESPACE: [char; 6] = [' ', '\t', '\n', '\r', '(', ')'];

pub fn is_ws(ch: char) -> bool {
    WHITESPACE.contains(&ch)
}

/// At least one whitespace repeated
pub fn ws1<'a, Input>(input: Input) -> IResult<Input, Input>
where
    Input: InputTakeAtPosition<Item = char>,
{
    take_while1(is_ws)(input)
}

/// Any amount of whitespace repeated
pub fn ws<'a, Input>(input: Input) -> IResult<Input, Input>
where
    Input: InputTakeAtPosition<Item = char>,
{
    take_while(is_ws)(input)
}
