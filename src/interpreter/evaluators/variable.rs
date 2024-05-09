//! Contains variable related structures

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::*,
    combinator::opt,
    multi::many1,
    sequence::{tuple, Tuple},
    Parser,
};

use crate::{interpreter::runtime::error::Error, parsers::*};

use super::{
    expression::{AtomPostfix, Expression},
    parsers::AstParseResult,
};

#[derive(Debug, Clone)]
/// Declared variable
pub struct VariableDecl {
    name: String,
    pub expression: Expression,
    line: usize,
    type_: VarType,
    life_time: Option<LifeTime>,
}

#[derive(Debug, Clone, Copy)]
pub enum VarType {
    VarVar,
    ConstVar,
    VarConst,
    ConstConst,
}

impl VariableDecl {
    pub fn eval(&self, args: PosWithInfo) -> Result<(), Error> {
        let interpreter = args.extra.0;
        let value = self.expression.eval(args)?;
        interpreter.state.add_var(
            &self.name,
            value.0.into_owned(),
            self.line,
            self.type_,
            self.life_time,
        );

        Ok(())
    }

    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        let var = || tag("var");
        let const_ = || tag("const");
        let eq = char('=');
        let const_const = tuple((const_(), ws1, const_())).map(|_| VarType::ConstConst);
        let const_var = tuple((const_(), ws1, var())).map(|_| VarType::ConstVar);
        let var_const = tuple((var(), ws1, const_())).map(|_| VarType::VarConst);
        let var_var = tuple((var(), ws1, var())).map(|_| VarType::VarVar);
        let type_ = alt((const_const, var_var, const_var, var_const));
        let identifier = identifier(LifeTime::parse);

        let line = input.line;

        // var ws+ var ws+ identifier life_time? ws* "=" ws* expr
        let (input, (type_, _, identifier, life_time, _, _, _, expression, _)) = (
            type_,
            ws1,
            identifier,
            opt(LifeTime::parse),
            ws,
            eq,
            ws,
            Expression::parse,
            end_of_statement,
        )
            .parse(input)?;

        let decl = Self {
            expression,
            name: identifier.input.to_string(),
            line,
            type_,
            life_time,
        };

        Ok((input, decl))
    }
}

#[derive(Debug, Clone)]
pub struct VarSet {
    name: String,
    postfix: Vec<AtomPostfix>,
    expression: Expression,
    line: usize,
}

impl VarSet {
    pub fn eval(&self, args: PosWithInfo) -> Result<(), Error> {
        let interpreter = args.extra.0;
        let value = self.expression.eval(args)?;
        interpreter.state.set_var(
            &self.name,
            args,
            &self.postfix,
            value.0.into_owned(),
            self.line,
        )?;
        Ok(())
    }

    pub fn parse(input_orig: PosWithInfo) -> AstParseResult<Self> {
        // ident ws* "=" ws* expr ws* !
        let eq = char('=');
        let mut identifier_full = identifier(LifeTime::parse);
        let (mut input, mut var_identifier) = identifier_full(input_orig)?;

        let mut postfix = None;
        if input_orig
            .extra
            .0
            .state
            .get_var(var_identifier.input)
            .is_none()
        {
            // try with postfix
            if let Ok((input_, identifier_postfix)) = identifier(alt((
                AtomPostfix::parse.map(|_| ()),
                LifeTime::parse.map(|_| ()),
            )))(input_orig)
            {
                if let Ok((input_, postfix_)) = many1(AtomPostfix::parse)(input_) {
                    var_identifier = identifier_postfix;
                    input = input_;
                    postfix = Some(postfix_);
                }
            }
        }
        let (input, (_, _, _, expression, _)) =
            (ws, eq, ws, Expression::parse, end_of_statement).parse(input)?;

        let decl = Self {
            expression,
            name: var_identifier.input.to_string(),
            line: var_identifier.line,
            postfix: postfix.unwrap_or_default(),
        };

        Ok((input, decl))
    }
}
