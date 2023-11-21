use std::borrow::Borrow;

use nom::{
    branch::*, bytes::complete::*, combinator::*, error::*, multi::*, number::complete::*,
    sequence::*, *,
};

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

/// Takes a chunk of code until the next whitespace
pub fn chunk(input: Position) -> PosResult<&str> {
    take_while(|ch| !is_ws(ch))
        .map(|slice: Position<'_>| slice.input)
        .parse(input)
}

pub fn identifier<'a, I, E, P, PO>(terminating_parser: P) -> impl Fn(I) -> IResult<I, I, E>
where
    I: InputTakeAtPosition<Item = char> + InputIter + InputTake + Borrow<str> + Copy + InputLength,
    E: ParseError<I>,
    P: Parser<I, PO, E> + Clone,
{
    move |input| {
        let terminating_parser = terminating_parser.clone();
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

pub enum LifeTime {
    Infinity,
    Seconds(f64),
    Lines(isize),
}

impl LifeTime {
    pub fn parse<'a, T: Clone>(input: Position<'a, T>) -> PosResult<Self, T> {
        let infinity = tag("Infinity").map(|_| LifeTime::Infinity);
        let seconds =
            terminated(double, character::complete::char('s')).map(|s| LifeTime::Seconds(s));
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
pub fn parse_isize<'a, T: Clone>(input: Position<T>) -> PosResult<'a, isize, T> {
    let input: Position<'_, T, &'a [u8]> = Position::from(input);

    // who even uses 128 bit pointers?
    let result = if cfg!(target_pointer_width = "64") {
        be_u64.map(|x| x as isize).parse(input)
    } else if cfg!(target_pointer_width = "32") {
        be_u32.map(|x| x as isize).parse(input)
    } else if cfg!(target_pointer_width = "128") {
        be_u128.map(|x| x as isize).parse(input)
    } else if cfg!(target_pointer_width = "16") {
        be_u16.map(|x| x as isize).parse(input)
    } else if cfg!(target_pointer_width = "8") {
        be_u8.map(|x| x as isize).parse(input)
    } else {
        unimplemented!("Unsupported pointer width")
    };

    let (input, result) = result.map_err(|err: Err<Error<_>>| {
        err.map_input(|i| <Position<_, &'a str> as From<Position<_, &'a [u8]>>>::from(i))
    })?;

    let input: Position<'_, T, &'a str> = Position::from(input);

    Ok((input, result))
}
