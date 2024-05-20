use std::sync::{Arc, Mutex};

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{consumed, fail, opt, recognize, verify},
    multi::many0,
    sequence::tuple,
    IResult, Parser,
};

use crate::{
    interpreter::evaluators::scope::scope,
    parsers::{identifier, types::Position, ws, ws1, PosWithInfo},
    runtime::{self, value::Value},
};

use super::{expression::Expression, parsers::AstParseResult, statement::Statement};

#[derive(Debug, Clone)]
pub struct When {
    expression: String,
    body: String,
    body_line: usize,
    // TODO: store the previous value in the state, since prev keyword is a thing
    prev_identifier_values: Arc<Mutex<Vec<Option<Value>>>>,
    identifiers: Vec<String>,
    // TODO: to prevent recursive invokes, make this vec later on
    else_when: Option<ElseWhen>,
}

#[derive(Debug, Clone)]
pub enum ElseWhen {
    When(Box<When>),
    Else { body: String, body_line: usize },
}

impl When {
    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        let when = tag("when");
        let else_ = || tag("else");

        fn bracket_start(input: PosWithInfo) -> IResult<PosWithInfo, PosWithInfo, ()> {
            tag("{")(input)
        }
        let expression = recognize(Expression::parser(Some(bracket_start)));
        let body = || recognize(scope);
        // TODO: probably don't make it recursive in call later
        let else_when =
            tuple((else_(), ws1, When::parse)).map(|(_, _, when)| ElseWhen::When(Box::new(when)));
        let else_ =
            tuple((else_(), consumed(ws1), body())).map(|(_, (ws_pos, _), body)| ElseWhen::Else {
                body: body.to_string(),
                body_line: ws_pos.line,
            });

        let (input, (_, _, expression, _, (body_consumed, body), _)) =
            tuple((when, ws1, expression, ws, consumed(body()), ws))(input)?;

        let body_line = body_consumed.line;

        let (input, else_when) = opt(alt((else_when, else_)))(input)?;

        // grab all identifiers
        // TODO: limit to current scope?
        let expr_identifier = identifier(fail::<_, (), ()>);

        let (_, identifiers) = many0(
            tuple((expr_identifier, ws)).map(|(i, _): (Position<_, _>, _)| i.to_string()),
        )(expression)
        .unwrap();

        let interpreter = input.extra.0;
        let values = identifiers.iter().map(|i| {
            interpreter
                .state
                .get_var(i)
                .map(|var| var.get_value().to_owned())
        });

        Ok((
            input,
            Self {
                expression: expression.to_string(),
                body: body.to_string(),
                body_line,
                prev_identifier_values: Arc::new(Mutex::new(values.collect())),
                identifiers,
                else_when,
            },
        ))
    }

    pub fn eval(&self, args: PosWithInfo) {
        args.extra.0.state.push_when(self);
    }

    pub fn eval_body(
        &self,
        args: PosWithInfo,
        var_name: &str,
        new_value: Value,
    ) -> Result<(), runtime::Error> {
        let Some(identifier_index) = self.identifiers.iter().position(|i| i == var_name) else {
            // not found
            return self.else_when_exec(args, false, var_name, new_value);
        };

        let mut prev_values = self.prev_identifier_values.lock().unwrap();
        let prev_value = &prev_values[identifier_index];

        let changed = match prev_value {
            Some(prev_value) => !prev_value.strict_eq(&new_value), // has it changed?
            None => true,                                          // declaration of new value
        };

        // finally, update
        prev_values[identifier_index] = Some(new_value.clone());

        drop(prev_values);

        if !changed {
            return self.else_when_exec(args, true, var_name, new_value);
        }

        // TODO: does position matter
        let code_with_pos = Position::new_with_extra(self.expression.as_str(), args.extra);
        let (_, expr) = Expression::parse(code_with_pos).unwrap();
        let value = expr.eval(args)?;

        if !bool::from(value.0.as_ref()) {
            return self.else_when_exec(args, true, var_name, new_value);
        }

        Self::body_exec(args, &self.body, self.body_line)
    }

    fn else_when_exec(
        &self,
        args: PosWithInfo,
        found: bool,
        var_name: &str,
        new_value: Value,
    ) -> Result<(), runtime::Error> {
        let Some(else_when) = &self.else_when else {
            return Ok(());
        };

        match else_when {
            ElseWhen::When(when) => when.eval_body(args, var_name, new_value),
            ElseWhen::Else { body, body_line } => {
                if !found {
                    return Ok(());
                }
                Self::body_exec(args, body, *body_line)
            }
        }
    }

    fn body_exec(args: PosWithInfo, body: &str, body_line: usize) -> Result<(), runtime::Error> {
        // parse and execute the body
        args.extra.0.state.push_scope(body_line);

        let mut code_with_pos = Position::new_with_extra(body, args.extra);

        let mut scope_count = 0usize;

        while let Ok((code_after, statement)) = Statement::parse(code_with_pos) {
            match statement {
                Statement::ScopeStart(_) => {
                    scope_count = scope_count.checked_add(1).expect("scope count overflow")
                }
                Statement::ScopeEnd(_) => {
                    scope_count -= 1;
                    if scope_count == 0 {
                        break;
                    }
                }
                _ => (),
            }

            code_with_pos = code_after;

            let ret = statement.eval(code_with_pos)?;
            if ret.return_value.is_some() {
                args.extra.0.state.pop_scope(code_with_pos.line);
                return Ok(());
            }
        }

        let (_, statements) = many0(Statement::parse)(code_with_pos).unwrap();
        for statement in statements {
            statement.eval(args)?;
        }

        Ok(())
    }
}

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
