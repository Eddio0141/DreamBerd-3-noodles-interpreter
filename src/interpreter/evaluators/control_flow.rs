use nom::{bytes::complete::tag, sequence::tuple};

use crate::parsers::{end_of_statement, ws, PosWithInfo};

use super::parsers::AstParseResult;

#[derive(Debug)]
pub struct Reverse;

impl Reverse {
    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        let reverse = tag("reverse");

        let (input, _) = tuple((reverse, ws, end_of_statement))(input)?;

        Ok((input, Self))
    }

    pub fn eval(&self, args: PosWithInfo) {
        args.extra.0.state.toggle_reverse();
    }
}
