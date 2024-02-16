use crate::{parsers::types::Position, Interpreter};

pub type AstParseResult<'input, O> = Result<
    (Position<'input, Interpreter>, O),
    nom::Err<nom::error::Error<Position<'input, Interpreter>>>,
>;
