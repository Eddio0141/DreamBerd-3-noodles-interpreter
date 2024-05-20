//! Contains variable related structures

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::*,
    combinator::{fail, opt, value},
    multi::many1,
    sequence::{tuple, Tuple},
    Parser,
};
use num_bigint::BigInt;
use num_traits::FromPrimitive;

use crate::{interpreter::runtime::error::Error, parsers::*, runtime::value::Value};

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
        interpreter.state.add_var_runtime(
            &self.name,
            value.0.into_owned(),
            self.line,
            self.type_,
            self.life_time,
            args,
        )?;

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
    expression: Option<Expression>,
    line: usize,
    op: VarSetOp,
}

#[derive(Debug, Clone)]
pub enum VarSetOp {
    // assign
    Equals,
    // logical
    And,
    Or,
    // arithmetic
    Add,
    AddOne,
    SubOne,
    Subtract,
    Multiply,
    Exponential,
    Divide,
    Modulo,
}

impl VarSet {
    // TODO: add returning value before change as expr
    // TODO: test above
    // TODO: prefix operation
    pub fn eval(&self, args: PosWithInfo) -> Result<(), Error> {
        let interpreter = args.extra.0;

        let expr = self
            .expression
            .as_ref()
            .map(|expr| expr.eval(args).map(|value| value.0.into_owned()));
        let value = || {
            interpreter
                .state
                .get_var(&self.name)
                .unwrap()
                .get_value()
                .to_owned()
        };

        let value = match self.op {
            VarSetOp::Equals => expr.unwrap()?,
            VarSetOp::And => Value::Boolean(value().into() && expr.unwrap()?.into()),
            VarSetOp::Or => Value::Boolean(value().into() || expr.unwrap()?.into()),
            VarSetOp::Add => (value() + expr.unwrap()?)?,
            VarSetOp::Subtract => (value() - expr.unwrap()?)?,
            VarSetOp::Multiply => (value() * expr.unwrap()?)?,
            VarSetOp::Exponential => value().pow(&expr.unwrap()?)?,
            VarSetOp::Divide => (value() / expr.unwrap()?)?,
            VarSetOp::Modulo => (value() % expr.unwrap()?)?,
            VarSetOp::AddOne => {
                let value = value();
                match value {
                    Value::Number(value) => Value::Number(value + 1.0),
                    Value::BigInt(value) => Value::BigInt(value + BigInt::from_u8(1).unwrap()),
                    _ => {
                        // TODO: is this behaviour accurate?
                        // TODO: error handle
                        let value = f64::try_from(value).unwrap();
                        Value::Number(value + 1.0)
                    }
                }
            }
            VarSetOp::SubOne => {
                let value = value();
                match value {
                    Value::Number(value) => Value::Number(value - 1.0),
                    Value::BigInt(value) => Value::BigInt(value - BigInt::from_u8(1).unwrap()),
                    _ => {
                        // TODO: is this behaviour accurate?
                        // TODO: error handle
                        let value = f64::try_from(value).unwrap();
                        Value::Number(value - 1.0)
                    }
                }
            }
        };

        interpreter
            .state
            .set_var(&self.name, args, &self.postfix, value, self.line)?;

        Ok(())
    }

    pub fn parse(input_orig: PosWithInfo) -> AstParseResult<Self> {
        // TODO: do i need lifetime check here
        let mut identifier_full = identifier(alt((
            LifeTime::parse.map(|_| ()),
            VarSetOp::parse.map(|_| ()),
        )));
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

        // normal assignment
        let (input, (_, op, _)) = (ws, VarSetOp::parse, ws).parse(input)?;

        // if operator depends on some existing variable, but variable doesn't exist, fail
        if !matches!(op, VarSetOp::Equals)
            && input.extra.0.state.get_var(var_identifier.input).is_none()
        {
            return fail(input);
        }

        // in case of ++ and --, no expression required
        // TODO: rewrite as a nom parser probably
        if matches!(op, VarSetOp::AddOne | VarSetOp::SubOne) {
            let decl = Self {
                expression: None,
                name: var_identifier.input.to_string(),
                line: var_identifier.line,
                postfix: postfix.unwrap_or_default(),
                op,
            };

            let (input, _) = end_of_statement(input)?;

            return Ok((input, decl));
        }

        // expression is required
        let (input, (expression, _)) = (Expression::parse, end_of_statement).parse(input)?;

        let decl = Self {
            expression: Some(expression),
            name: var_identifier.input.to_string(),
            line: var_identifier.line,
            postfix: postfix.unwrap_or_default(),
            op,
        };

        Ok((input, decl))
    }
}

impl VarSetOp {
    // TODO: test those
    fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        alt((
            value(VarSetOp::Equals, char('=')),
            value(VarSetOp::And, tag("&=")),
            value(VarSetOp::Or, tag("|=")),
            value(VarSetOp::Add, tag("+=")),
            value(VarSetOp::AddOne, tag("++")),
            value(VarSetOp::SubOne, tag("--")),
            value(VarSetOp::Subtract, tag("-=")),
            value(VarSetOp::Multiply, tag("*=")),
            value(VarSetOp::Exponential, tag("^=")),
            value(VarSetOp::Divide, tag("/=")),
            value(VarSetOp::Modulo, tag("%=")),
        ))(input)
    }
}
