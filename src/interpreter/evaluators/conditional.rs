use nom::{
    bytes::complete::tag,
    combinator::{opt, verify},
    multi::many0,
    sequence::tuple,
    IResult, Parser,
};

use crate::{
    parsers::{ws, ws1, PosWithInfo},
    runtime,
};

use super::{expression::Expression, parsers::AstParseResult, statement::Statement};

// a vec of statements instead of a string like a function
// this is because if statement shouldn't be able to introduce side effects like
// defining a variable named `{` or `if` and such
type Body = Vec<Statement>;

#[derive(Debug)]
pub struct If {
    expression: Expression,
    body: Body,
    else_ifs: Vec<ElseIf>,
    else_: Option<Body>,
}

#[derive(Debug)]
struct ElseIf {
    expression: Expression,
    body: Body,
}

impl If {
    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        fn bracket_start(input: PosWithInfo) -> IResult<PosWithInfo, PosWithInfo, ()> {
            tag("{")(input)
        }
        let expression = || Expression::parser(Some(bracket_start));
        let else_ = || tuple((ws, tag("else")));
        let body = |input| {
            let mut bracket_start =
                verify(Statement::parse, |s| matches!(s, Statement::ScopeStart(_)));

            let (mut input, bracket_start) = bracket_start(input)?;

            let mut depth = 0usize;
            let mut statements = vec![bracket_start];

            loop {
                let (input_, statement) = Statement::parse(input)?;
                input = input_;
                match statement {
                    Statement::ScopeStart(_) => depth += 1,
                    Statement::ScopeEnd(_) => {
                        if depth == 0 {
                            break;
                        }
                        depth -= 1;
                    }
                    _ => {}
                }
                statements.push(statement);
            }

            Ok((input, statements))
        };
        let if_ = || {
            tuple((tag("if"), ws1, expression(), ws, body))
                .map(|(_, _, expr, _, body)| (expr, body))
        };
        let else_if = tuple((else_(), ws1, if_()))
            .map(|(_, _, (expression, body))| ElseIf { expression, body });
        let else_ = tuple((else_(), ws1, body)).map(|(_, _, expr)| expr);

        let (input, ((expression, body), else_ifs, else_)) =
            tuple((if_(), many0(else_if), opt(else_)))(input)?;

        Ok((
            input,
            If {
                expression,
                body,
                else_,
                else_ifs,
            },
        ))
    }

    pub fn eval(&self, args: PosWithInfo) -> Result<(), runtime::Error> {
        let if_expr = |expr: &Expression| Ok(expr.eval(args)?.0.as_ref().into());
        let exec_body = |body: &Body| {
            for statement in body {
                statement.eval(args)?;
            }
            Ok(())
        };

        if if_expr(&self.expression)? {
            return exec_body(&self.body);
        }

        for else_if in &self.else_ifs {
            if if_expr(&else_if.expression)? {
                return exec_body(&else_if.body);
            }
        }

        if let Some(else_) = &self.else_ {
            return exec_body(else_);
        }

        Ok(())
    }
}
