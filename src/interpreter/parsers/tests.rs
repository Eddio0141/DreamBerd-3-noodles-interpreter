use nom::{bytes::complete::tag, InputTake, Slice};

use crate::parsers::types::Position;

use super::types::calc_line_column;

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
