use crate::{parsers::types::Position, Interpreter};

pub type AstParseResult<'input, 'b, O> = Result<
    (Position<'input, Interpreter<'b>>, O),
    nom::Err<nom::error::Error<Position<'input, Interpreter<'b>>>>,
>;
