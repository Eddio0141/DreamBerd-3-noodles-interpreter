use std::collections::HashMap;

use nom::{
    branch::alt, bytes::complete::tag, character::complete, multi::separated_list0,
    sequence::tuple, IResult, Parser,
};

use crate::{
    impl_parser,
    parsers::{types::Position, ws},
    runtime::value::{Object, Value},
    Interpreter,
};

use super::expression::Expression;

#[derive(Debug, Clone)]
/// Represents an array.
/// - [Mozilla documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array)
pub struct ArrayInitialiser(Vec<Expression>);

impl_parser!(
    ArrayInitialiser,
    input,
    {
        fn expr_term<'a, 'b>(
            input: Position<'a, Interpreter<'b>>,
        ) -> IResult<Position<'a, Interpreter<'b>>, Position<'a, Interpreter<'b>>, ()> {
            alt((tag(","), tag("]")))(input)
        }

        let brace_start = complete::char('[');
        let brace_end = complete::char(']');
        let comma = complete::char(',');
        let item = Expression::parser(Some(expr_term));
        let item = tuple((ws, item, ws)).map(|(_, item, _)| item);
        let items = separated_list0(comma, item);

        let (input, (_, items, _)) = tuple((brace_start, items, brace_end))(input)?;

        Ok((input, Self(items)))
    },
    self,
    eval_args,
    {
        // TODO: implement constructors and fix this
        let mut props = HashMap::new();

        if let Some(first) = self.0.first() {
            props.insert("-1".to_string(), first.eval(eval_args)?.0.into_owned());
        }
        for i in 1..self.0.len() {
            let item = &self.0[i];
            props.insert((i - 1).to_string(), item.eval(eval_args)?.0.into_owned());
        }

        let obj = Object::new(props);

        Ok(obj.into())
    },
    Value
);
