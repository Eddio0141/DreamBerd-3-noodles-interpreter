use crate::{parsers::types::Position, Interpreter};

pub type AstParseResult<'a, O> = Result<
    (Position<'a, &'a Interpreter<'a>>, O),
    nom::Err<nom::error::Error<Position<'a, &'a Interpreter<'a>>>>,
>;
