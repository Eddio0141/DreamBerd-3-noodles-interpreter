use std::collections::HashMap;

use nom::{
    character::complete::char, combinator::cut, multi::separated_list0, sequence::tuple, Parser,
};

use crate::{
    parsers::{identifier, types::Position, ws, PosWithInfo},
    runtime::{
        value::{Object, Value},
        Error,
    },
};

use super::{expression::Expression, parsers::AstParseResult};

#[derive(Debug, Clone)]
/// Represents an object initialiser.
/// - [Mozilla documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer)
pub struct ObjectInitialiser(HashMap<String, Expression>);

impl ObjectInitialiser {
    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        let brace_start = char('{');
        let brace_end = char('}');
        let colon = || char(':');
        let identifier = identifier(colon()).map(|id: Position<_, _>| id.to_string());
        let comma = char(',');
        let property = tuple((ws, identifier, ws, colon(), ws, cut(Expression::parse)))
            .map(|(_, id, _, _, _, expr)| (id, expr));

        let (input, (_, properties, _, _)) =
            tuple((brace_start, separated_list0(comma, property), ws, brace_end))(input)?;

        // unique properties, if duplicate, last one is used
        let properties = properties.into_iter().collect();

        Ok((input, Self(properties)))
    }

    pub fn eval(&self, eval_args: PosWithInfo) -> Result<Value, Error> {
        // TODO: implement constructor
        let mut obj = HashMap::new();
        for (key, value) in self.0.iter() {
            obj.insert(key.to_string(), value.eval(eval_args)?.0.into_owned());
        }
        let obj = Object::new(obj);

        Ok(obj.into())
    }
}
