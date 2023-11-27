use nom::{
    bytes::complete::tag,
    character::{self, complete::satisfy},
    InputTake, InputTakeAtPosition, Slice,
};

use crate::parsers::{types::Position, LifeTime};

use super::{identifier, types::calc_line_column, ws_char};

#[test]
fn split_at_position() {
    let input = Position::new("foo bar");

    // split like i.. and ..i
    let (l, r) = input
        .split_at_position::<_, ()>(|c| c == ' ')
        .unwrap();
    assert_eq!(l.input, " bar");
    assert_eq!(l.line, 1);
    assert_eq!(l.column, 4);
    assert_eq!(l.index, 3);

    assert_eq!(r.input, "foo");
    assert_eq!(r.line, 1);
    assert_eq!(r.column, 1);
    assert_eq!(r.index, 0);
}

#[test]
fn tag_parse() {
    let input = Position::new("tag");
    let (input, tag) = tag::<_, _, ()>("tag")(input).unwrap();
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
    let input = Position::new("fooobar");
    let (l, r) = input.take_split(4);

    assert_eq!(r.input, "fooo");
    assert_eq!(r.line, 1);
    assert_eq!(r.column, 1);
    assert_eq!(r.index, 0);

    assert_eq!(l.input, "bar");
    assert_eq!(l.line, 1);
    assert_eq!(l.column, 5);
}

#[test]
fn identifier_term_at_parser() {
    let input = Position::new("foo1 bar");
    let mut identifier = identifier(character::complete::char::<_, ()>('1'));

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
    let mut identifier = identifier(character::complete::char::<_, ()>('1'));

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
    let mut identifier = identifier(character::complete::char::<_, ()>('1'));

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
fn identifier_empty() {
    let input = Position::new("");
    let mut identifier = identifier(character::complete::char::<_, ()>('1'));

    let result = identifier(input);
    assert!(result.is_err());
}

#[test]
fn identifier_start_at_ws() {
    let input = Position::new(" ");
    let mut identifier = identifier(character::complete::char::<_, ()>('1'));

    let result = identifier(input);
    assert!(result.is_err());
}

#[test]
fn identifier_start_at_term() {
    let input = Position::new("1");
    let mut identifier = identifier(character::complete::char::<_, ()>('1'));

    let result = identifier(input);
    assert!(result.is_err());
}

#[test]
fn satisfy_parse() {
    let input = Position::new("foo bar");
    let (input, c) = satisfy::<_, _, ()>(|c| c == 'f')(input).unwrap();
    assert_eq!(c, 'f');
    assert_eq!(input.input, "oo bar");
    assert_eq!(input.line, 1);
    assert_eq!(input.column, 2);
    assert_eq!(input.index, 1);
}

#[test]
fn ws_char_parse() {
    let input = Position::new(" foo bar");
    let (input, _) = ws_char::<_, ()>(input).unwrap();
    assert_eq!(input.input, "foo bar");
    assert_eq!(input.line, 1);
    assert_eq!(input.column, 2);
    assert_eq!(input.index, 1);
}

#[test]
fn life_time_neg_line() {
    let input = Position::new("<-5>");
    let (_, life_time) = LifeTime::parse(input).unwrap();
    assert_eq!(life_time, LifeTime::Lines(-5));
}

#[test]
fn life_time_pos_line() {
    let input = Position::new("<5>");
    let (_, life_time) = LifeTime::parse(input).unwrap();
    assert_eq!(life_time, LifeTime::Lines(5));
}

#[test]
fn life_time_zero_line() {
    let input = Position::new("<0>");
    let (_, life_time) = LifeTime::parse(input).unwrap();
    assert_eq!(life_time, LifeTime::Lines(0));
}

#[test]
fn life_time_seconds() {
    let input = Position::new("<10.5s>");
    let (_, life_time) = LifeTime::parse(input).unwrap();
    assert_eq!(life_time, LifeTime::Seconds(10.5));
}

#[test]
fn life_time_neg_seconds_fail() {
    let input = Position::new("<-10.5s>");
    assert!(LifeTime::parse(input).is_err());
}

#[test]
fn life_time_zero_seconds() {
    let input = Position::new("<0s>");
    let (_, life_time) = LifeTime::parse(input).unwrap();
    assert_eq!(life_time, LifeTime::Seconds(0.));
}

#[test]
fn life_time_infinity() {
    let input = Position::new("<Infinity>");
    let (_, life_time) = LifeTime::parse(input).unwrap();
    assert_eq!(life_time, LifeTime::Infinity);
}
