//! Contains utilities for parsing the source code into an AST.

#[cfg(test)]
mod tests;

use nom::{branch::alt, bytes::complete::*, character, combinator::*, multi::*, sequence::*, *};

use crate::{
    interpreter::parsers::*,
    parsers::types::{PosResult, Position},
};

pub fn till_term<'a>(input: Position<'a>) -> PosResult<()> {
    let str = |input: Position<'a>| -> PosResult<'a, ()> {
        let quote = alt((
            character::complete::char::<_, nom::error::Error<_>>('"'),
            character::complete::char('\''),
        ));
        let (input, mut left_quotes) = many0(quote)(input).unwrap();
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

        let (input, _) = verify(take(left_quotes.len()), |s: &Position| {
            for (i, input_c) in s.input.chars().enumerate() {
                if input_c != left_quotes[i] {
                    return false;
                }
            }
            true
        })
        .map(|_| ())
        .parse(input)?;

        Ok((input, ()))
    };

    let (input, _) = many0(alt((str, is_not("!").map(|_| ()))))(input)?;

    Ok((input, ()))
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

pub fn function_expression(input: Position) -> PosResult<(Option<Vec<Position>>, Position)> {
    let arrow = tag("=>");
    let comma = || character::complete::char(',');
    let arg = identifier(tuple((comma(), ws)));
    let args = separated_list0(comma(), arg);

    tuple((opt(args), ws1, arrow, till_term))
        .map(|(args, _, arrow, _)| (args, arrow))
        .parse(input)
}
