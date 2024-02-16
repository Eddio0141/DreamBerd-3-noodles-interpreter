//! Contains utilities for parsing the source code into an AST.

#[cfg(test)]
mod tests;

use std::fmt::Debug;

use nom::{
    branch::alt, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*,
    *,
};

use crate::{
    interpreter::parsers::*,
    parsers::types::{PosResult, Position},
};

// TODO: wait for fix, or make issue if not
#[allow(clippy::let_and_return)]
pub fn till_term<'a>(input: Position<'a>) -> PosResult<'a, Position<'a>> {
    let str = |input: Position<'a>| -> PosResult<'a, Position> {
        let quote = alt((char('"'), char('\'')));
        let (input, mut left_quotes) = many1(quote)(input)?;
        let (input, _) = take_till::<_, _, ()>(|c| c == left_quotes[0])(input).unwrap();
        // since we checking right to left now
        left_quotes.reverse();

        let result = verify(take(left_quotes.len()), |s: &Position| {
            for (i, input_c) in s.input.chars().enumerate() {
                if input_c != left_quotes[i] {
                    return false;
                }
            }
            true
        })
        .parse(input);
        result
    };

    let (mut input, statement_chunks) = (many0(alt((str, is_not("!"))))).parse(input)?;

    // trim the "!"
    for ch in input.input.chars() {
        if ch != '!' {
            break;
        }

        let (input_new, _) = take::<_, _, ()>(1usize)(input).unwrap();
        input = input_new;
    }

    Ok((input, statement_chunks[0]))
}

/// Parses a variable declaration
/// # Note
/// - The expression parser is expected to handle all the way including the `!` terminator
/// # Returns
/// (var_decl_pos, identifier, life_time, expression_parser_output)
pub fn var_decl<'a, P, O: Debug>(
    mut expression_parser: P,
) -> impl FnMut(Position<'a>) -> PosResult<'a, (Position<'a>, Position<'a>, Option<LifeTime>, O)>
where
    P: Parser<Position<'a>, O, nom::error::Error<Position<'a>>>,
{
    move |input_original: Position| {
        let var = || tag("var");
        let eq = char('=');
        let identifier = identifier(LifeTime::parse);

        // var ws+ var ws+ identifier life_time? ws* "=" ws* expr "!"
        //
        // var var func<-5> = arg1, arg2, ... => (expression or something)!
        let (input, (_, _, _, _, identifier, life_time, _, _, _)) = tuple((
            var(),
            ws1,
            var(),
            ws1,
            identifier,
            opt(LifeTime::parse),
            ws,
            eq,
            ws,
        ))(input_original)?;

        let (input, expr) = expression_parser.parse(input)?;

        Ok((input, (input_original, identifier, life_time, expr)))
    }
}

/// Parses a function expression
/// # Example
/// - ` => statement!`
/// - `arg1,arg2 , arg3 =>statement!`
///
/// # Returns
/// - Arguments of the function with their identifiers
/// - Position of where the statement starts
pub fn function_expression(input: Position) -> PosResult<(Vec<Position>, Position)> {
    let arrow = || tag("=>");
    let comma = || char(',');
    let arg = identifier(comma());
    // either an arrow start (meaning no args) or a list of args
    let args = alt((
        value(Vec::new(), tuple((ws, arrow()))),
        tuple((separated_list0(tuple((ws, comma(), ws)), arg), ws, arrow()))
            .map(|(args, _, _)| args),
    ));

    tuple((args, ws, till_term))
        .map(|(args, _, expr)| (args, expr))
        .parse(input)
}
