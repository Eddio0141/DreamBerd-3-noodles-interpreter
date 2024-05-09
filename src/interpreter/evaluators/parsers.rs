use crate::parsers::PosWithInfo;

pub type AstParseResult<'input, O> =
    Result<(PosWithInfo<'input>, O), nom::Err<nom::error::Error<PosWithInfo<'input>>>>;
