use std::{fmt::Debug, ops::RangeFrom};

use self::types::*;
use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, error::*, multi::*,
    number::complete::*, sequence::*, *,
};

#[cfg(test)]
mod tests;
pub mod types;

pub static WHITESPACE: [char; 6] = [' ', '\t', '\n', '\r', '(', ')'];

pub fn is_ws(ch: char) -> bool {
    WHITESPACE.contains(&ch)
}

/// Tries to parse a single whitespace character
/// # Note
/// - It also handles comments
pub fn ws_char<'a, I, E>(input: I) -> IResult<I, (), E>
where
    I: Slice<RangeFrom<usize>>
        + InputIter<Item = char>
        + InputTake
        + InputLength
        + Clone
        + InputTakeAtPosition<Item = char>
        + Compare<&'a str>,
    E: ParseError<I>,
{
    value((), tuple((satisfy(is_ws), opt(comment_line))))(input)
}

/// At least one whitespace repeated
pub fn ws1<'a, I>(input: I) -> IResult<I, ()>
where
    I: InputIter<Item = char>
        + InputLength
        + InputTake
        + Clone
        + InputTakeAtPosition<Item = char>
        + Slice<RangeFrom<usize>>
        + Compare<&'a str>,
{
    value((), many1_count(ws_char))(input)
}

/// Any amount of whitespace repeated
/// # Returns
/// - The amount of whitespace
pub fn ws_count<'a, I>(input: I) -> IResult<I, usize>
where
    I: InputIter<Item = char>
        + InputLength
        + InputTake
        + Clone
        + InputTakeAtPosition<Item = char>
        + Slice<RangeFrom<usize>>
        + Compare<&'a str>,
{
    many0_count(ws_char)(input)
}

/// Any amount of whitespace repeated
pub fn ws<'a, I, E>(input: I) -> IResult<I, (), E>
where
    I: InputLength
        + InputIter<Item = char>
        + InputTake
        + Clone
        + InputTakeAtPosition<Item = char>
        + Slice<RangeFrom<usize>>
        + Compare<&'a str>,
    E: ParseError<I>,
{
    value((), many0_count(ws_char))(input)
}

/// Parses and ignores a comment
pub fn comment_line<'a, I, E>(input: I) -> IResult<I, (), E>
where
    I: InputTake
        + InputIter<Item = char>
        + InputLength
        + InputTakeAtPosition<Item = char>
        + Clone
        + Compare<&'a str>,
    E: ParseError<I>,
{
    let start = tag("//");
    let end = take_while(|ch| ch != '\n');
    value((), tuple((start, end)))(input)
}

/// Takes a chunk of code until the next whitespace
pub fn chunk<I, E>(input: I) -> IResult<I, I, E>
where
    I: InputLength + InputIter<Item = char> + InputTake + Clone + InputTakeAtPosition<Item = char>,
    E: ParseError<I>,
{
    take_while(|ch| !is_ws(ch))(input)
}

/// Takes a chunk of code until the next whitespace
/// # Note
/// - This version will not include the terminator and stops before it or the whitespace
pub fn terminated_chunk<I, E>(input: I) -> IResult<I, I, E>
where
    I: InputLength + InputIter<Item = char> + InputTake + Clone + InputTakeAtPosition<Item = char>,
    E: ParseError<I>,
{
    take_while(|ch| !is_ws(ch) && ch != '!')(input)
}

/// Gets the identifier
/// # Arguments
/// - `terminating_parser`: A parser that terminates the identifier. If this is hit or a whitespace is hit, then the identifier is terminated
///
/// # Returns
/// - The identifier
pub fn identifier<'a, I, E, P, PO>(mut terminating_parser: P) -> impl FnMut(I) -> IResult<I, I, E>
where
    I: InputIter<Item = char>
        + InputTake
        + Copy
        + InputLength
        + Slice<RangeFrom<usize>>
        + Compare<&'a str>
        + InputTakeAtPosition<Item = char>,
    E: ParseError<I>,
    P: Parser<I, PO, E>,
{
    move |input_original| {
        let mut input = input_original;
        let mut take_count = 0;
        let mut first_ch = true;
        loop {
            if input.input_len() == 0 {
                break;
            }

            // is it terminating
            // note: terminating parser can be the name of the identifier as well
            if peek(ws_char::<_, ()>)(input).is_ok()
                || (!first_ch && terminating_parser.parse(input).is_ok())
            {
                // don't consume
                break;
            }

            let (_, take_length) = take::<_, _, ()>(1usize)(input).unwrap();
            let take_length = take_length.input_len();

            take_count += take_length;
            input = input.take_split(take_length).0;
            first_ch = false;
        }

        // don't allow empty identifiers
        if take_count == 0 {
            return Err(Err::Error(E::from_error_kind(
                input_original,
                ErrorKind::TakeWhile1,
            )));
        }

        Ok(input_original.take_split(take_count))
    }
}

#[derive(Debug, PartialEq)]
pub enum LifeTime {
    Infinity,
    Seconds(f64),
    Lines(isize),
}

impl LifeTime {
    pub fn parse<'a, 'b, T: Debug>(input: Position<'a, 'b, T>) -> PosResult<'a, 'b, Self, T> {
        let infinity = tag("Infinity").map(|_| LifeTime::Infinity);
        let seconds = map_opt(terminated(double, char('s')), |s| {
            if s.is_sign_negative() {
                None
            } else {
                Some(LifeTime::Seconds(s))
            }
        });
        let lines = parse_isize.map(LifeTime::Lines);
        delimited(char('<'), alt((infinity, seconds, lines)), char('>'))(input)
    }
}

/// Tries to parse an `isize` from the input
/// - This properly handles the target pointer width depending on the platform
pub fn parse_isize<'a, 'b, T: Debug>(input: Position<'a, 'b, T>) -> PosResult<'a, 'b, isize, T> {
    let negative = char::<Position<_>, _>('-');
    tuple((opt(negative).map(|v| v.is_some()), digit1))
        .map(|(neg, digits)| {
            let digits = digits.input.parse::<isize>().unwrap();
            if neg {
                digits.saturating_neg()
            } else {
                digits
            }
        })
        .parse(input)
}

/// End of statement including the whitespace before it
pub fn end_of_statement<'a, I, E>(input: I) -> IResult<I, (), E>
where
    I: InputIter<Item = char>
        + Clone
        + InputLength
        + Slice<RangeFrom<usize>>
        + InputTake
        + InputTakeAtPosition<Item = char>
        + Compare<&'a str>,
    E: ParseError<I>,
{
    let end = many1(char('!'));
    value((), tuple((ws, end)))(input)
}
