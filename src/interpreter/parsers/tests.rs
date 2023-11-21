use nom::{
    bytes::complete::tag,
    character::{self, complete::satisfy},
    InputTake, Slice,
};

use crate::parsers::types::Position;

use super::{identifier, types::calc_line_column, ws_char};

#[test]
fn tag_parse() {
    let input = Position::new("tag");
    let (input, tag) = tag::<_, _, nom::error::Error<_>>("tag")(input).unwrap();
    assert!(
        input.input.is_empty(),
        "input is not empty, input: {:?}, tag: {:?}",
        input,
        tag
    );
    assert_eq!(input.line, 1);
    assert_eq!(input.column, 4);
    assert_eq!(input.index, 3);
}

#[test]
fn slice_to() {
    let input = Position::new("slice");
    let input = input.slice(..3);
    assert_eq!(input.input, "sli");
    assert_eq!(input.line, 1);
    assert_eq!(input.column, 1);
    assert_eq!(input.index, 0);
}

#[test]
fn slice_from() {
    let input = Position::new("slice");
    let input = input.slice(3..);
    assert_eq!(input.input, "ce");
    assert_eq!(input.line, 1);
    assert_eq!(input.column, 4);
    assert_eq!(input.index, 3);
}

#[test]
fn calc_line_column_test() {
    let input = "foo bar\nbaz";
    let (line, column) = calc_line_column(&input);
    assert_eq!(line, 1);
    assert_eq!(column, 3);
}

#[test]
fn take_split() {
    let input = Position::new("foobar");
    let (l, r) = input.take_split(3);

    assert_eq!(r.input, "foo");
    assert_eq!(r.line, 1);
    assert_eq!(r.column, 1);
    assert_eq!(r.index, 0);

    assert_eq!(l.input, "bar");
    assert_eq!(l.line, 1);
    assert_eq!(l.column, 4);
}

#[test]
fn identifier_term_at_parser() {
    let input = Position::new("foo1 bar");
    let mut identifier = identifier(character::complete::char::<_, nom::error::Error<_>>('1'));

    let (input, identifier) = identifier(input).unwrap();

    assert_eq!(identifier.input, "foo");
    assert_eq!(identifier.line, 1);
    assert_eq!(identifier.column, 1);
    assert_eq!(identifier.index, 0);

    assert_eq!(input.input, "1 bar");
    assert_eq!(input.line, 1);
    assert_eq!(input.column, 4);
    assert_eq!(input.index, 3);
}

#[test]
fn identifier_term_at_ws() {
    let input = Position::new("foo 1bar");
    let mut identifier = identifier(character::complete::char::<_, nom::error::Error<_>>('1'));

    let (input, identifier) = identifier(input).unwrap();

    assert_eq!(identifier.input, "foo");
    assert_eq!(identifier.line, 1);
    assert_eq!(identifier.column, 1);
    assert_eq!(identifier.index, 0);

    assert_eq!(input.input, " 1bar");
    assert_eq!(input.line, 1);
    assert_eq!(input.column, 4);
    assert_eq!(input.index, 3);
}

#[test]
fn identifier_no_ws_no_term() {
    let input = Position::new("foobar");
    let mut identifier = identifier(character::complete::char::<_, nom::error::Error<_>>('1'));

    let (input, identifier) = identifier(input).unwrap();

    assert_eq!(identifier.input, "foobar");
    assert_eq!(identifier.line, 1);
    assert_eq!(identifier.column, 1);
    assert_eq!(identifier.index, 0);

    assert_eq!(input.input, "");
    assert_eq!(input.line, 1);
    assert_eq!(input.column, 7);
    assert_eq!(input.index, 6);
}

#[test]
fn satisfy_parse() {
    let input = Position::new("foo bar");
    let (input, c) = satisfy::<_, _, nom::error::Error<_>>(|c| c == 'f')(input).unwrap();
    assert_eq!(c, 'f');
    assert_eq!(input.input, "oo bar");
    assert_eq!(input.line, 1);
    assert_eq!(input.column, 2);
    assert_eq!(input.index, 1);
}

#[test]
fn ws_char_parse() {
    let input = Position::new(" foo bar");
    let (input, _) = ws_char::<_, nom::error::Error<_>>(input).unwrap();
    assert_eq!(input.input, "foo bar");
    assert_eq!(input.line, 1);
    assert_eq!(input.column, 2);
    assert_eq!(input.index, 1);
}
