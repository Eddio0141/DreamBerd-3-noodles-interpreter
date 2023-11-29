use crate::{parsers::types::Position, Interpreter};

pub type AstParseResult<'a, 'b, 'c, O> = Result<
    (Position<'a, 'b, Interpreter<'c>>, O),
    nom::Err<nom::error::Error<Position<'a, 'b, Interpreter<'c>>>>,
>;
