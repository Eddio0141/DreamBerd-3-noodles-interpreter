use std::{
    ops::{Range, RangeFrom, RangeTo},
    str::{CharIndices, Chars},
};

use nom::{
    error::{ErrorKind, ParseError},
    *,
};

/// Position information for parsing and an extra field for additional information
#[derive(Debug, Clone, Copy)]
pub struct Position<'a, T = ()>
where
    T: Clone,
{
    pub line: usize,
    pub column: usize,
    pub index: usize,
    pub input: &'a str,
    pub extra: T,
}

impl<T: Clone> InputLength for Position<'_, T> {
    fn input_len(&self) -> usize {
        self.input.input_len()
    }
}

impl<'a, T: Clone> InputIter for Position<'a, T> {
    type Item = char;
    type Iter = CharIndices<'a>;
    type IterElem = Chars<'a>;

    fn iter_indices(&self) -> Self::Iter {
        self.input.char_indices()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.input.chars()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        for (o, c) in self.input.char_indices() {
            if predicate(c) {
                return Some(o);
            }
        }
        None
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        let mut cnt = 0;
        for (index, _) in self.input.char_indices() {
            if cnt == count {
                return Ok(index);
            }
            cnt += 1;
        }
        if cnt == count {
            return Ok(self.input.len());
        }
        Err(Needed::Unknown)
    }
}

impl<T: Clone> InputTake for Position<'_, T> {
    fn take(&self, count: usize) -> Self {
        let mut new = *self;
        new.input = self.input.take(count);
        new
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        self.left_right_split(&self.input[..count], &self.input[count..], count)
    }
}

fn calc_line_column(input: &str) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for c in input.chars() {
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

impl<T: Clone> InputTakeAtPosition for Position<'_, T> {
    type Item = char;

    fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.find(predicate) {
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => Ok(self.left_right_split(&self.input[..i], &self.input[i..], i)),
            None => Err(Err::Incomplete(Needed::new(1))),
        }
    }

    fn split_at_position1<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.find(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(*self, e))),
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => Ok(self.left_right_split(&self.input[..i], &self.input[i..], i)),
            None => Err(Err::Incomplete(Needed::new(1))),
        }
    }

    fn split_at_position_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.find(predicate) {
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => Ok(self.left_right_split(&self.input[..i], &self.input[i..], i)),
            // the end of slice is a char boundary
            None => Ok(self.left_right_split(
                &self.input[self.input.len()..],
                &self.input[..self.input.len()],
                0,
            )),
        }
    }

    fn split_at_position1_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.find(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(*self, e))),
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => Ok(self.left_right_split(&self.input[..i], &self.input[i..], i)),
            None => {
                if self.input.is_empty() {
                    Err(Err::Error(E::from_error_kind(*self, e)))
                } else {
                    // the end of slice is a char boundary
                    Ok(self.left_right_split(
                        &self.input[self.input.len()..],
                        &self.input[..self.input.len()],
                        0,
                    ))
                }
            }
        }
    }
}

impl<T: Clone> Slice<RangeFrom<usize>> for Position<'_, T> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        let (left, right) = (&self.input[..range.start], &self.input[range.start..]);

        Self {
            line: self.line,
            column: self.column,
            index: self.index,
            input: right,
            extra: self.extra.clone(),
        }
    }
}

impl<T: Clone> Slice<RangeTo<usize>> for Position<'_, T> {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        let (left, right) = (&self.input[..range.end], &self.input[range.end..]);
        let (line, column) = calc_line_column(right);

        Self {
            line: self.line - line,
            column: self.column - column,
            index: self.index - right.len(),
            input: left,
            extra: self.extra.clone(),
        }
    }
}

impl<T: Clone> Slice<Range<usize>> for Position<'_, T> {
    fn slice(&self, range: Range<usize>) -> Self {
        // position would be left + right
        let (left, right) = (&self.input[range], &self.input[range.end..]);
        let (line, column) = calc_line_column(right);

        Self {
            line: self.line - line,
            column: self.column - column,
            index: self.index - right.len(),
            input: left,
            extra: self.extra.clone(),
        }
    }
}

impl<T: Clone> Compare<&str> for Position<'_, T> {
    fn compare(&self, t: &str) -> CompareResult {
        self.input.compare(t)
    }

    fn compare_no_case(&self, t: &str) -> CompareResult {
        self.input.compare_no_case(t)
    }
}

impl<'a> Position<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
            input,
            extra: (),
        }
    }
}

impl<'a, E: Clone> Position<'a, E> {
    fn left_right_split(&self, left: &'a str, right: &'a str, len: usize) -> (Self, Self) {
        let (line, column) = calc_line_column(left);
        (
            Self {
                line: self.line,
                column: self.column,
                index: self.index,
                input: left,
                extra: self.extra.clone(),
            },
            Self {
                line: self.line + line,
                column: self.column + column,
                index: self.index + len,
                input: right,
                extra: self.extra.clone(),
            },
        )
    }

    pub fn new_with_extra(input: &'a str, extra: E) -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
            input,
            extra,
        }
    }
}

pub type PosResult<'a, O, T = (), E = nom::error::Error<Position<'a, T>>> =
    Result<(Position<'a, T>, O), nom::Err<E>>;
