use nom::{
    branch::alt, bytes::complete::tag, character::complete, multi::separated_list0,
    sequence::tuple, IResult, Parser,
};

use crate::{
    impl_parser,
    parsers::{types::Position, ws},
    runtime::{stdlib::array, value::Value},
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
        fn expr_term(
            input: Position<Interpreter>,
        ) -> IResult<Position<Interpreter>, Position<Interpreter>, ()> {
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
        let args = self
            .0
            .iter()
            .map(|expr| expr.eval(eval_args))
            .collect::<Result<Vec<_>, _>>()?;
        array::constructor(eval_args.1.extra, args)
    },
    Value
);
