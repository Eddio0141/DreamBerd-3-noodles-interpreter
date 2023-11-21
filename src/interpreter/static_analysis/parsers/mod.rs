//! Contains utilities for parsing the source code into an AST.

#[cfg(test)]
mod tests;

use nom::{branch::alt, bytes::complete::*, character, combinator::*, multi::*, sequence::*, *};

use crate::{
    interpreter::parsers::*,
    parsers::types::{PosResult, Position},
};

pub fn till_term<'a>(input: Position<'a>) -> PosResult<Position> {
    let str = |input: Position<'a>| -> PosResult<'a, Position> {
        let quote = alt((
            character::complete::char::<_, nom::error::Error<_>>('"'),
            character::complete::char('\''),
        ));
        let (input, mut left_quotes) = many1(quote)(input)?;
        let (input, _) = many0(verify(
            tuple((
                take::<_, Position, nom::error::Error<_>>(1usize),
                peek(take(1usize)),
            )),
            |(f, s)| {
                !(f.input.chars().next().unwrap() != '\\'
                    && s.input.chars().next().unwrap() == left_quotes[0])
            },
        ))(input)
        .unwrap();
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

    let (input, a) = many0(alt((str, is_not("!"))))(input)?;

    Ok((input, a[0]))
}

/// Parses a variable declaration
/// # Returns
/// (var_decl_pos, identifier, life_time, expression_parser_output)
pub fn var_decl<'a, P, O>(
    mut expression_parser: P,
) -> impl FnMut(Position<'a>) -> PosResult<'a, (Position, Position, Option<LifeTime>, O)>
where
    P: Parser<Position<'a>, O, nom::error::Error<Position<'a>>>,
{
    move |input_original: Position| {
        let var = || tag("var");
        let eq = character::complete::char('=');
        let statement_end = character::complete::char('!');
        let identifier = identifier(LifeTime::parse);

        // var ws+ var ws+ identifier life_time? ws* "=" ws* expr "!"
        //
        // var var func<-5> = arg1, arg2, ... => (expression or something)!
        let (input, (_, _, _, _, identifier, life_time, _, _, _)) = ((
            var(),
            ws1,
            var(),
            ws1,
            identifier,
            opt(LifeTime::parse),
            ws,
            eq,
            ws,
        ))
            .parse(input_original)?;

        let (input, expr) = expression_parser.parse(input)?;
        let (input, _) = statement_end(input)?;

        Ok((input, (input_original, identifier, life_time, expr)))
    }
}

/// Parses a function expression
/// # Example
/// ` => statement!`
/// `arg1,arg2 , arg3 =>statement!`
///
/// # Returns
/// - Arguments of the function with their identifiers
/// - Position of where the statement starts
pub fn function_expression(input: Position) -> PosResult<(Vec<Position>, Position)> {
    let arrow = tag("=>");
    let comma = || character::complete::char(',');
    let arg = identifier(comma());
    let args = separated_list0(comma(), arg);

    tuple((args, ws, arrow, till_term))
        .map(|(args, _, _, expr)| (args, expr))
        .parse(input)
}
