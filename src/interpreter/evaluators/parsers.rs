use std::ops::RangeFrom;

use crate::interpreter::parsers::*;
use nom::{
    bytes::complete::{take_while, take_while1},
    character::complete,
    error::ErrorKind,
    multi::many0_count,
    AsChar, IResult, InputIter, InputLength, InputTake, Slice, UnspecializedInput,
};

pub fn identifier_optional_term<'a, Input>(
    term_char: char,
) -> impl FnMut(Input) -> IResult<Input, &'a str>
where
    Input: InputIter<Item = char> + InputTake + Copy + Into<&'a str>,
{
    // ident -> ws
    // or
    // ident -> term_char -> ws

    move |input| {
        let mut input_iter = input.iter_indices();
        let first_ch_err = move || {
            Err(nom::Err::Failure(nom::error::Error::new(
                input.clone(),
                ErrorKind::TakeWhile1,
            )))
        };

        match input_iter.next() {
            Some((_, first_ch)) => {
                if WHITESPACE.contains(&first_ch) {
                    return first_ch_err();
                }
            }
            None => return first_ch_err(),
        }

        for (i, ch) in input_iter {
            if term_char != ch && !is_ws(ch) {
                continue;
            }

            let (left, right) = input.take_split(i);
            return Ok((left, right.into()));
        }

        todo!()
    }
}

pub fn term<'a, Input>(input: Input) -> IResult<Input, usize>
where
    Input: InputIter<Item = char> + Slice<RangeFrom<usize>> + Clone + InputLength,
{
    let term = complete::char('!');
    many0_count(term)(input)
}

pub fn equals<'a, Input, Error>(input: Input) -> IResult<Input, char>
where
    Input: Slice<RangeFrom<usize>> + InputIter,
    <Input as InputIter>::Item: AsChar,
{
    complete::char('=')(input)
}
