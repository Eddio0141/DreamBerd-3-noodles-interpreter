use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::fail, sequence::tuple,
};

use crate::{
    parsers::{identifier, types::Position},
    runtime, Interpreter,
};

use super::{parsers::AstParseResult, EvalArgs};

#[derive(Debug, Default)]
struct Class {
    name: String,
}

impl Class {
    pub fn parse(input: Position<Interpreter>) -> AstParseResult<Self> {
        let class = alt((tag("className"), tag("class")));
        let name = identifier(fail::<_, (), _>);
        let open = char('{');
        let close = char('}');

        let (input, (_, name, _, _)) = tuple((class, name, open, close))(input)?;

        todo!()
    }

    pub fn eval(args: EvalArgs) -> Result<(), runtime::Error> {
        todo!()
    }
}
