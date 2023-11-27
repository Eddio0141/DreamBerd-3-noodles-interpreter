//! Contains variable related structures

use nom::bytes::complete::tag;
use nom::character;
use nom::combinator::opt;
use nom::sequence::Tuple;

use crate::parsers::types::Position;
use crate::parsers::{identifier, ws, LifeTime};

use crate::interpreter::runtime::error::Error;
use crate::Interpreter;

use super::expression::Expression;
use super::parsers::AstParseResult;

#[derive(Debug, Clone)]
/// Declared variable
pub struct VariableDecl {
    name: String,
    expression: Expression,
}

impl VariableDecl {
    pub fn eval(&self, interpreter: &Interpreter) -> Result<(), Error> {
        let value = self.expression.eval(interpreter)?;
        interpreter.state.add_var(&self.name, value.0.into_owned());

        Ok(())
    }

    pub fn parse<'a>(input: Position<'a, &'a Interpreter<'a>>) -> AstParseResult<'a, Self> {
        let var = || tag("var");
        let eq = character::complete::char('=');
        let identifier = identifier(LifeTime::parse);
        // var ws+ var ws+ identifier life_time? ws* "=" ws* expr
        let (input, (_, _, _, _, identifier, _, _, _, _, expression)) = (
            var(),
            ws,
            var(),
            ws,
            identifier,
            opt(LifeTime::parse),
            ws,
            eq,
            ws,
            Expression::parse,
        )
            .parse(input)?;

        let decl = Self {
            expression,
            name: identifier.input.to_string(),
        };

        Ok((input, decl))
    }
}

#[derive(Debug, Clone)]
pub struct VarSet {
    name: String,
    expression: Expression,
}

impl VarSet {
    pub fn eval<'a>(&'a self, interpreter: &Interpreter<'a>) -> Result<(), Error> {
        let value = self.expression.eval(interpreter)?;
        interpreter.state.set_var(&self.name, value.0.into_owned());
        Ok(())
    }

    pub fn parse(code: &str) -> AstParseResult<Self> {
        // let funcs = input.static_analysis.current_funcs();

        // let identifier = identifier_optional_term('=');

        // let (input, identifier_peek) = peek(identifier)(input)?;

        // if let Some((_, func)) = funcs.get_key_value(identifier_peek) {
        //     if func.arg_count != 0 {
        //         // not a variable declaration
        //         return FunctionCall::parse(input)
        //             .map(|(input, func)| (input, Statement::FunctionCall(func)));
        //     }
        // }

        // // ident ~ ws* ~ "=" ~ ws* ~ expr ~ ws* ~ !
        // let (input, (_, _, _, _, expression, _, _)) = (
        //     identifier,
        //     ws,
        //     equals::<_, nom::error::Error<ParserInput>>,
        //     ws,
        //     Expression::parse,
        //     ws,
        //     term,
        // )
        //     .parse(input)?;

        // let var_set = Self {
        //     name: identifier_peek.to_string(),
        //     expression: expression.into(),
        // };

        // Ok((input, Statement::VarSet(var_set)))
        todo!()
    }
}
