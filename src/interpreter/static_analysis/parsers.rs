//! Contains utilities for parsing the source code into an AST.

#[cfg(test)]
mod tests;

use nom::{
    branch::alt, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*,
    *,
};

use crate::{
    interpreter::parsers::*,
    parsers::types::{PosResult, Position},
};

use super::HoistedVarInfo;

// TODO: wait for fix, or make issue if not
#[allow(clippy::let_and_return)]
pub fn till_term<'a>(input: Position<'a>) -> PosResult<'a, ()> {
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

    many1(alt((str, is_not("!")))).map(|_| ()).parse(input)
}

/// Parses a variable declaration
/// # Note
/// - The expression parser is expected to handle all the way including the `!` terminator
/// # Returns
/// (identifier, hoisted_line, expression_parser_output)
pub fn var_decl(input_original: Position<'_>) -> PosResult<'_, HoistedVarInfo> {
    let var = || tag("var");
    let const_ = || tag("const");
    let eq = char('=');
    let identifier = identifier(LifeTime::parse);
    let const_const = tuple((const_(), ws1, const_())).map(|_| ());
    let const_var = tuple((const_(), ws1, var())).map(|_| ());
    let var_const = tuple((var(), ws1, const_())).map(|_| ());
    let var_var = tuple((var(), ws1, var())).map(|_| ());
    let var_decl = alt((const_const, var_var, const_var, var_const));

    let mut expr = many_till(
        tuple((
            alt((
                function_expression.map(|_| ()),
                terminated_chunk.map(|_| ()),
            )),
            ws,
        )),
        end_of_statement,
    );

    // var ws+ var ws+ identifier life_time? ws* "=" ws* expr "!"
    //
    // var var func<-5> = arg1, arg2, ... => (expression or something)!
    let (input, (_, _, identifier, life_time, _, _, _)) =
        tuple((var_decl, ws1, identifier, LifeTime::parse, ws, eq, ws))(input_original)?;

    let expr_index = input.index;

    // TODO: it would be better to use Expression::parse, but that has a dependency on Interpreter
    // for now, I have a scuffed shitty dumbed down version of that used here, but it should be
    // generic enough to be shared in runtime parsing, and static analysis parsing here
    let (input, _) = expr(input)?;

    let decl_line = input_original.line;
    let hoisted_line = match life_time {
        LifeTime::Infinity => decl_line, // positive infinity
        LifeTime::Seconds(_) => decl_line,
        LifeTime::Lines(lines) => {
            // only go backwards if lines is negative
            if lines.is_negative() {
                decl_line.saturating_add_signed(lines)
            } else {
                decl_line
            }
        }
    };

    let var_info = HoistedVarInfo {
        identifier: identifier.input.to_string(),
        hoisted_line,
        expr_index,
    };

    Ok((input, var_info))
}

/// Parses a function expression
/// # Example
/// - ` => statement!`
/// - `arg1,arg2 , arg3 =>statement!`
pub fn function_expression(input: Position) -> PosResult<()> {
    let arrow = || tag("=>");
    let comma = || char(',');
    let arg = identifier(comma());
    // either an arrow start (meaning no args) or a list of args
    let args = alt((
        value(Vec::new(), tuple((ws, arrow()))),
        tuple((separated_list0(tuple((ws, comma(), ws)), arg), ws, arrow()))
            .map(|(args, _, _)| args),
    ));

    tuple((args, ws, till_term)).map(|_| ()).parse(input)
}
