use std::{
    iter::{Copied, Enumerate},
    marker::PhantomData,
    ops::{Range, RangeFrom, RangeTo},
    slice::Iter,
    str::{CharIndices, Chars, FromStr},
};

use nom::{
    error::{ErrorKind, ParseError},
    *,
};

/// Position information for parsing and an extra field for additional information
/// # Note
/// - The position points to the start of the input
/// - As the input is parsed and sliced, the position will be updated
#[derive(Debug, Clone, Copy)]
pub struct Position<'a, T: Clone = (), I = &'a str> {
    pub line: usize,
    pub column: usize,
    pub index: usize,
    pub input: I,
    pub extra: T,
    _phantom: PhantomData<&'a I>,
}

impl<T: Clone, I: InputLength> InputLength for Position<'_, T, I> {
    fn input_len(&self) -> usize {
        self.input.input_len()
    }
}

impl<'a, T: Clone> InputIter for Position<'a, T, &'a str> {
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

impl<'a, T: Clone> InputIter for Position<'a, T, &'a [u8]> {
    type Item = u8;
    type Iter = Enumerate<Self::IterElem>;
    type IterElem = Copied<Iter<'a, u8>>;

    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.input.iter().copied()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.input.iter().position(|b| predicate(*b))
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.input.len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.input.len()))
        }
    }
}

impl<'a, T: Clone, I> InputTake for Position<'_, T, I>
where
    I: InputTake + Slice<RangeTo<usize>> + Slice<RangeFrom<usize>> + AsChars<'a>,
{
    fn take(&self, count: usize) -> Self {
        let mut new = *self;
        new.input = self.input.take(count);
        new
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        self.left_right_split(self.input.slice(..count), self.input.slice(count..), count)
    }
}

fn calc_line_column<'a, I: AsChars<'a>>(input: I) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for c in input.as_chars() {
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

trait AsChars<'a> {
    fn as_chars(&'a self) -> impl Iterator<Item = char>;
}

impl<'a> AsChars<'a> for &str {
    fn as_chars(&'a self) -> impl Iterator<Item = char> {
        self.chars()
    }
}

impl<'a> AsChars<'a> for &'a [u8] {
    fn as_chars(&'a self) -> impl Iterator<Item = char> {
        std::str::from_utf8(self).unwrap().chars()
    }
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

impl<T: Clone, I> Slice<RangeFrom<usize>> for Position<'_, T, I>
where
    I: Slice<RangeTo<usize>> + Slice<RangeFrom<usize>>,
{
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        let (left, right) = (
            self.input.slice(..range.start),
            self.input.slice(range.start..),
        );

        Self {
            line: self.line,
            column: self.column,
            index: self.index,
            input: right,
            extra: self.extra.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Clone, I> Slice<RangeTo<usize>> for Position<'_, T, I>
where
    I: Slice<RangeTo<usize>> + Slice<RangeFrom<usize>> + AsChars<'a> + InputLength,
{
    fn slice(&self, range: RangeTo<usize>) -> Self {
        let (left, right) = (self.input.slice(..range.end), self.input.slice(range.end..));
        let (line, column) = calc_line_column(right);

        Self {
            line: self.line - line,
            column: self.column - column,
            index: self.index - right.input_len(),
            input: left,
            extra: self.extra.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Clone, I> Slice<Range<usize>> for Position<'_, T, I>
where
    I: Slice<Range<usize>> + Slice<RangeFrom<usize>> + AsChars<'a> + InputLength,
{
    fn slice(&self, range: Range<usize>) -> Self {
        // position would be left + right
        let (left, right) = (self.input.slice(range), self.input.slice(range.end..));
        let (line, column) = calc_line_column(right);

        Self {
            line: self.line - line,
            column: self.column - column,
            index: self.index - right.input_len(),
            input: left,
            extra: self.extra.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Clone> From<Position<'_, T, &'a str>> for Position<'a, T, &'a [u8]> {
    fn from(input: Position<'_, T, &'a str>) -> Self {
        Self {
            line: input.line,
            column: input.column,
            index: input.index,
            input: input.input.as_bytes(),
            extra: input.extra,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Clone> From<Position<'_, T, &'a [u8]>> for Position<'a, T, &'a str> {
    fn from(input: Position<'_, T, &'a [u8]>) -> Self {
        Self {
            line: input.line,
            column: input.column,
            index: input.index,
            input: std::str::from_utf8(input.input).unwrap(),
            extra: input.extra,
            _phantom: PhantomData,
        }
    }
}

impl<T: Clone, I: Compare<I2>, I2> Compare<I2> for Position<'_, T, I> {
    fn compare(&self, t: I2) -> CompareResult {
        self.input.compare(t)
    }

    fn compare_no_case(&self, t: I2) -> CompareResult {
        self.input.compare_no_case(t)
    }
}

impl<T: Clone, I: Offset> Offset for Position<'_, T, I> {
    fn offset(&self, second: &Self) -> usize {
        self.input.offset(&second.input)
    }
}

impl<T: FromStr, E: Clone> ParseTo<T> for Position<'_, E> {
    fn parse_to(&self) -> Option<T> {
        self.input.parse_to()
    }
}

impl<E: Clone> AsBytes for Position<'_, E> {
    fn as_bytes(&self) -> &[u8] {
        self.input.as_bytes()
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
            _phantom: PhantomData,
        }
    }
}

impl<'a, E: Clone, I: AsChars<'a>> Position<'a, E, I> {
    fn left_right_split(&self, left: I, right: I, len: usize) -> (Self, Self) {
        let (line, column) = calc_line_column(left);
        (
            Self {
                line: self.line,
                column: self.column,
                index: self.index,
                input: left,
                extra: self.extra.clone(),
                _phantom: PhantomData,
            },
            Self {
                line: self.line + line,
                column: self.column + column,
                index: self.index + len,
                input: right,
                extra: self.extra.clone(),
                _phantom: PhantomData,
            },
        )
    }

    pub fn new_with_extra(input: I, extra: E) -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
            input,
            extra,
            _phantom: PhantomData,
        }
    }
}

pub type PosResult<'a, O, T = (), I = &'a str, E = nom::error::Error<Position<'a, T>>> =
    Result<(Position<'a, T, I>, O), nom::Err<E>>;
