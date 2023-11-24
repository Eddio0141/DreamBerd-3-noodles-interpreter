use std::{borrow::Borrow, fmt::Debug, ops::RangeFrom};

use self::types::*;
use nom::{
    branch::*,
    bytes::complete::*,
    character::complete::{digit1, satisfy},
    combinator::*,
    error::*,
    multi::{many0_count, many1},
    number::complete::*,
    sequence::*,
    *,
};

#[cfg(test)]
mod tests;
pub mod types;

pub static WHITESPACE: [char; 6] = [' ', '\t', '\n', '\r', '(', ')'];

pub fn is_ws(ch: char) -> bool {
    WHITESPACE.contains(&ch)
}

/// Tries to parse a single whitespace character
pub fn ws_char<I, E>(input: I) -> IResult<I, char, E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = char>,
    E: ParseError<I>,
{
    satisfy(is_ws)(input)
}

/// At least one whitespace repeated
pub fn ws1<I>(input: I) -> IResult<I, ()>
where
    I: InputIter<Item = char> + InputLength + InputTake + Clone + InputTakeAtPosition<Item = char>,
{
    take_while1(is_ws).map(|_| ()).parse(input)
}

/// Any amount of whitespace repeated
/// # Returns
/// - The amount of whitespace
pub fn ws_count<I>(input: I) -> IResult<I, usize>
where
    I: InputIter<Item = char>
        + InputLength
        + InputTake
        + Clone
        + InputTakeAtPosition<Item = char>
        + Slice<RangeFrom<usize>>,
{
    many0_count(ws_char)(input)
}

/// Any amount of whitespace repeated
/// # Returns
/// - Slice of the whitespace
pub fn ws<I>(input: I) -> IResult<I, I>
where
    I: InputLength + InputIter<Item = char> + InputTake + Clone + InputTakeAtPosition<Item = char>,
{
    take_while(is_ws)(input)
}

/// Takes a chunk of code until the next whitespace
pub fn chunk<'a, I, E>(input: I) -> IResult<I, I, E>
where
    I: InputLength + InputIter<Item = char> + InputTake + Clone + InputTakeAtPosition<Item = char>,
    E: ParseError<I>,
{
    take_while(|ch| !is_ws(ch))(input)
}

/// Gets the identifier
/// # Arguments
/// - `terminating_parser`: A parser that terminates the identifier. If this is hit or a whitespace is hit, then the identifier is terminated
///
/// # Returns
/// - The identifier
pub fn identifier<I, E, P, PO>(mut terminating_parser: P) -> impl FnMut(I) -> IResult<I, I, E>
where
    I: InputTakeAtPosition<Item = char>
        + InputIter<Item = char>
        + InputTake
        + Borrow<str>
        + Copy
        + InputLength
        + Slice<RangeFrom<usize>>,
    E: ParseError<I>,
    P: Parser<I, PO, E>,
{
    move |input_original| {
        let mut input = input_original;
        let mut take_count = 0usize;
        loop {
            if input.input_len() == 0 {
                break;
            }

            // is it terminating?
            if matches!(terminating_parser.parse(input), Ok(_))
                || matches!(peek(ws_char::<_, nom::error::Error<_>>)(input), Ok(_))
            {
                // don't consume
                break;
            }

            take_count += 1;
            input = input.take_split(1).0;
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
    pub fn parse<'a, T: Copy + Debug>(input: Position<'a, T>) -> PosResult<Self, T> {
        let infinity = tag("Infinity").map(|_| LifeTime::Infinity);
        let seconds = map_opt(terminated(double, character::complete::char('s')), |s| {
            if s.is_sign_negative() {
                None
            } else {
                Some(LifeTime::Seconds(s))
            }
        });
        let lines = parse_isize.map(|l| LifeTime::Lines(l));
        delimited(
            character::complete::char('<'),
            alt((infinity, seconds, lines)),
            character::complete::char('>'),
        )(input)
    }
}

/// Tries to parse an `isize` from the input
/// - This properly handles the target pointer width depending on the platform
pub fn parse_isize<'a, T: Copy + Debug>(input: Position<'a, T>) -> PosResult<'a, isize, T> {
    let negative = character::complete::char::<Position<_>, _>('-');
    tuple((opt(negative).map(|v| v.is_some()), digit1))
        .map(|(neg, digits)| {
            let digits = isize::from_str_radix(digits.input, 10).unwrap();
            if neg {
                digits.saturating_neg()
            } else {
                digits
            }
        })
        .parse(input)
}

/// End of statement including the whitespace before it
pub fn end_of_statement<I>(input: I) -> IResult<I, ()>
where
    I: InputIter<Item = char>
        + Clone
        + InputLength
        + Slice<RangeFrom<usize>>
        + InputTake
        + InputTakeAtPosition<Item = char>,
{
    let end = many1(character::complete::char('!'));
    tuple((ws_count, end)).map(|_| ()).parse(input)
}
