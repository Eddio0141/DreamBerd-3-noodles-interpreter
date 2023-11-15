use nom::{bytes::complete::take_till, IResult};

pub static WHITESPACE: [char; 6] = [' ', '\t', '\n', '\r', '(', ')'];

fn is_ws(ch: char) -> bool {
    WHITESPACE.contains(&ch)
}

pub fn identifier<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    take_till(is_ws)(input)
}
