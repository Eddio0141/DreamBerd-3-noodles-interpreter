use nom::{
    branch::alt, bytes::complete::tag, character::complete, multi::separated_list0,
    sequence::tuple, IResult, Parser,
};

use crate::{
    parsers::{ws, PosWithInfo},
    runtime::{stdlib::array, value::Value, Error},
};

use super::{expression::Expression, parsers::AstParseResult};

#[derive(Debug, Clone)]
/// Represents an array.
/// - [Mozilla documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array)
pub struct ArrayInitialiser(Vec<Expression>);

impl ArrayInitialiser {
    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        fn expr_term(input: PosWithInfo) -> IResult<PosWithInfo, PosWithInfo, ()> {
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
    }

    pub fn eval(&self, eval_args: PosWithInfo) -> Result<Value, Error> {
        let args = self
            .0
            .iter()
            .map(|expr| expr.eval(eval_args))
            .collect::<Result<Vec<_>, _>>()?;
        array::constructor(eval_args.extra.0, args)
    }
}
