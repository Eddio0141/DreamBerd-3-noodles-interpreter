use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    iter::{Copied, Enumerate},
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
pub struct Position<'input, T = (), I: ?Sized = str> {
    pub line: usize,
    pub column: usize,
    pub index: usize,
    pub input: &'input I,
    pub extra: &'input T,
}

impl<'input, T> Display for Position<'input, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.input)
    }
}

impl<'input, T, I> Debug for Position<'input, T, I>
where
    I: ?Sized + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Position")
            .field("line", &self.line)
            .field("column", &self.column)
            .field("index", &self.index)
            .field("input", &self.input)
            .finish()
    }
}

impl<'input, T, I: ?Sized> Clone for Position<'input, T, I> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'input, T, I: ?Sized> Copy for Position<'input, T, I> {}

impl<'input, T, I: ?Sized> InputLength for Position<'input, T, I>
where
    &'input I: InputLength + 'input,
{
    fn input_len(&self) -> usize {
        self.input.input_len()
    }
}

impl<'input, T> InputIter for Position<'input, T, str> {
    type Item = char;
    type Iter = CharIndices<'input>;
    type IterElem = Chars<'input>;

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

impl<'input, T> InputIter for Position<'input, T, [u8]> {
    type Item = u8;
    type Iter = Enumerate<Self::IterElem>;
    type IterElem = Copied<Iter<'input, u8>>;

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

impl<'input, T, I: ?Sized> InputTake for Position<'input, T, I>
where
    &'input I: AsChars + InputLength + InputTake + Slice<RangeTo<usize>> + Slice<RangeFrom<usize>>,
{
    fn take(&self, count: usize) -> Self {
        let mut new = *self;
        new.input = &self.input.take(count);
        new
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let input_len = self.input_len();
        if count > input_len {
            panic!("count({count}) is larger than length({input_len})");
        }

        let (left, right) = (&self.input.slice(..count), &self.input.slice(count..));
        let (left, right) = self.left_right_split(left, right, count);
        (right, left)
    }
}

/// Calculates how many lines and columns the input produces
pub(super) fn calc_line_column<'input, I: ?Sized>(input: &'input I) -> (usize, usize)
where
    &'input I: AsChars,
{
    let mut line = 0;
    let mut column = 0;
    for c in input.as_chars() {
        if c == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
    (line, column)
}

pub trait AsChars {
    fn as_chars(&self) -> impl Iterator<Item = char>;
}

impl AsChars for &str {
    fn as_chars(&self) -> impl Iterator<Item = char> {
        self.chars()
    }
}

impl AsChars for &[u8] {
    fn as_chars(&self) -> impl Iterator<Item = char> {
        std::str::from_utf8(self).unwrap().chars()
    }
}

impl<T: Debug> InputTakeAtPosition for Position<'_, T> {
    type Item = char;

    fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.find(predicate) {
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => {
                let (left, right) = self.left_right_split(&self.input[..i], &self.input[i..], i);
                Ok((right, left))
            }
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
            Some(i) => {
                let (left, right) = self.left_right_split(&self.input[..i], &self.input[i..], i);
                Ok((right, left))
            }
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
        let (left, right) = match self.input.find(predicate) {
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => self.left_right_split(&self.input[..i], &self.input[i..], i),
            // the end of slice is a char boundary
            None => self.left_right_split(
                &self.input[..self.input.len()],
                &self.input[self.input.len()..],
                0,
            ),
        };

        Ok((right, left))
    }

    fn split_at_position1_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let (left, right) = match self.input.find(predicate) {
            Some(0) => return Err(Err::Error(E::from_error_kind(*self, e))),
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => self.left_right_split(&self.input[..i], &self.input[i..], i),
            None => {
                if self.input.is_empty() {
                    return Err(Err::Error(E::from_error_kind(*self, e)));
                } else {
                    // the end of slice is a char boundary
                    self.left_right_split(
                        &self.input[..self.input.len()],
                        &self.input[self.input.len()..],
                        0,
                    )
                }
            }
        };

        Ok((right, left))
    }
}

impl<'input, T, I: ?Sized> Slice<RangeFrom<usize>> for Position<'input, T, I>
where
    &'input I: Slice<RangeTo<usize>> + Slice<RangeFrom<usize>> + AsChars + InputLength + 'input,
{
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        let (left, right) = (
            self.input.slice(..range.start),
            self.input.slice(range.start..),
        );
        let (line, column) = calc_line_column(left);

        Self {
            line: self.line + line,
            column: self.column + column,
            index: self.index + left.input_len(),
            input: right,
            extra: self.extra,
        }
    }
}

impl<'input, T, I: ?Sized> Slice<RangeTo<usize>> for Position<'input, T, I>
where
    &'input I: Slice<RangeTo<usize>> + Slice<RangeFrom<usize>> + AsChars + InputLength + 'input,
{
    fn slice(&self, range: RangeTo<usize>) -> Self {
        let left = self.input.slice(..range.end);

        Self {
            line: self.line,
            column: self.column,
            index: self.index,
            input: left,
            extra: self.extra,
        }
    }
}

impl<'input, T, I: ?Sized> Slice<Range<usize>> for Position<'input, T, I>
where
    &'input I: Slice<Range<usize>> + Slice<RangeFrom<usize>> + AsChars + InputLength + 'input,
{
    fn slice(&self, range: Range<usize>) -> Self {
        // position would be left + right
        let (left, right) = (
            self.input.slice(range.clone()),
            self.input.slice(range.end..),
        );
        let (line, column) = calc_line_column(right);

        Self {
            line: self.line - line,
            column: self.column - column,
            index: self.index - right.input_len(),
            input: left,
            extra: self.extra,
        }
    }
}

impl<'input, T> From<Position<'input, T, &'input str>> for Position<'input, T, [u8]> {
    fn from(input: Position<'input, T, &'input str>) -> Self {
        Self {
            line: input.line,
            column: input.column,
            index: input.index,
            input: input.input.as_bytes(),
            extra: input.extra,
        }
    }
}

impl<'input, T> From<Position<'input, T, [u8]>> for Position<'input, T, str> {
    fn from(input: Position<'input, T, [u8]>) -> Self {
        Self {
            line: input.line,
            column: input.column,
            index: input.index,
            input: std::str::from_utf8(input.input).unwrap(),
            extra: input.extra,
        }
    }
}

impl<'input, T> From<Position<'input, T, str>> for &'input str {
    fn from(value: Position<'input, T, str>) -> Self {
        value.input
    }
}

impl<'input, I, I2, T> Compare<I2> for Position<'input, T, I>
where
    I: ?Sized,
    &'input I: Compare<I2>,
{
    fn compare(&self, t: I2) -> CompareResult {
        self.input.compare(t)
    }

    fn compare_no_case(&self, t: I2) -> CompareResult {
        self.input.compare_no_case(t)
    }
}

impl<'input, T, I> Offset for Position<'input, T, I>
where
    I: ?Sized,
    &'input I: Offset + 'input,
{
    fn offset(&self, second: &Self) -> usize {
        self.input.offset(&second.input)
    }
}

impl<T: FromStr, E> ParseTo<T> for Position<'_, E> {
    fn parse_to(&self) -> Option<T> {
        self.input.parse_to()
    }
}

impl<E> AsBytes for Position<'_, E> {
    fn as_bytes(&self) -> &[u8] {
        self.input.as_bytes()
    }
}

impl<E> Borrow<str> for Position<'_, E, str> {
    fn borrow(&self) -> &str {
        self.input
    }
}

impl<'input, E> FindSubstring<&'input str> for Position<'input, E, str> {
    fn find_substring(&self, substr: &'input str) -> Option<usize> {
        self.input.find(substr)
    }
}

impl<'input> Position<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
            input,
            extra: &(),
        }
    }
}

impl<'input, T, I: ?Sized> Position<'input, T, I>
where
    &'input I: AsChars,
{
    /// Splits the input into two positions
    fn left_right_split(&self, left: &'input I, right: &'input I, len: usize) -> (Self, Self) {
        let (line, column) = calc_line_column(left);
        (
            Self {
                line: self.line,
                column: self.column,
                index: self.index,
                input: left,
                extra: self.extra,
            },
            Self {
                line: self.line + line,
                column: self.column + column,
                index: self.index + len,
                input: right,
                extra: self.extra,
            },
        )
    }

    pub fn new_with_extra(input: &'input I, extra: &'input T) -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
            input,
            extra,
        }
    }
}

pub type PosResult<'input, O, T = (), I = str, E = nom::error::Error<Position<'input, T, I>>> =
    Result<(Position<'input, T, I>, O), nom::Err<E>>;
