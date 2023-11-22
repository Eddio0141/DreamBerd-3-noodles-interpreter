//! Contains variable related structures

use crate::parsers::types::Position;

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

    pub fn parse<'a>(input: Position<&Interpreter>) -> AstParseResult<'a, Self> {
        // let funcs = code.static_analysis.current_funcs();

        // if let Some((_, func)) = funcs.get_key_value("var") {
        //     if func.arg_count != 0 {
        //         // not a variable declaration
        //         return FunctionCall::parse(code)
        //             .map(|(left, func)| (left, Statement::FunctionCall(func)));
        //     }
        // }

        // // n
        // let var = || tag("var");
        // let identifier = identifier(life_time);
        // var ws+ var ws+ identifier
        // let (input, (_, _, _, _, identifier, life_time)) =
        //     (var(), ws, var(), ws, identifier, opt(life_time)).parse(input)?;

        // let decl = Self {
        //     expression: expression.into(),
        //     name: identifier.to_string(),
        // };

        // // input.code = code;

        // Ok((input, Statement::VariableDecl(decl)))
        todo!()
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
