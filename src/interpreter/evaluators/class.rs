use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::fail, sequence::tuple,
};

use crate::{
    parsers::{identifier, PosWithInfo},
    runtime,
};

use super::parsers::AstParseResult;

#[derive(Debug, Default)]
struct Class {
    name: String,
}

impl Class {
    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        let class = alt((tag("className"), tag("class")));
        let name = identifier(fail::<_, (), _>);
        let open = char('{');
        let close = char('}');

        let (input, (_, name, _, _)) = tuple((class, name, open, close))(input)?;

        todo!()
    }

    pub fn eval(args: PosWithInfo) -> Result<(), runtime::Error> {
        todo!()
    }
}
