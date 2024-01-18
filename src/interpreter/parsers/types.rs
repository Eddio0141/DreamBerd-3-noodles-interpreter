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
pub struct Position<'a, 'b, T = (), I: ?Sized = str> {
    pub line: usize,
    pub column: usize,
    pub index: usize,
    pub input: &'a I,
    pub extra: &'b T,
}

impl<'a, 'b, T> Display for Position<'a, 'b, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.input)
    }
}

impl<'a, 'b, T, I> Debug for Position<'a, 'b, T, I>
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

impl<'a, 'b, T, I: ?Sized> Clone for Position<'a, 'b, T, I> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, 'b, T, I: ?Sized> Copy for Position<'a, 'b, T, I> {}

impl<'a, 'b, T, I: ?Sized> InputLength for Position<'a, 'b, T, I>
where
    &'a I: InputLength + 'a,
{
    fn input_len(&self) -> usize {
        self.input.input_len()
    }
}

impl<'a, 'b, T> InputIter for Position<'a, 'b, T, str> {
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

impl<'a, 'b, T> InputIter for Position<'a, 'b, T, [u8]> {
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

impl<'a, 'b, T, I: ?Sized> InputTake for Position<'a, 'b, T, I>
where
    &'a I: AsChars + InputLength + InputTake + Slice<RangeTo<usize>> + Slice<RangeFrom<usize>>,
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
pub(super) fn calc_line_column<'a, I: ?Sized>(input: &'a I) -> (usize, usize)
where
    &'a I: AsChars,
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

impl<T: Debug> InputTakeAtPosition for Position<'_, '_, T> {
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

impl<'a, 'b, T, I: ?Sized> Slice<RangeFrom<usize>> for Position<'a, 'b, T, I>
where
    &'a I: Slice<RangeTo<usize>> + Slice<RangeFrom<usize>> + AsChars + InputLength + 'a,
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

impl<'a, 'b, T, I: ?Sized> Slice<RangeTo<usize>> for Position<'a, 'b, T, I>
where
    &'a I: Slice<RangeTo<usize>> + Slice<RangeFrom<usize>> + AsChars + InputLength + 'a,
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

impl<'a, 'b, T, I: ?Sized> Slice<Range<usize>> for Position<'a, 'b, T, I>
where
    &'a I: Slice<Range<usize>> + Slice<RangeFrom<usize>> + AsChars + InputLength + 'a,
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

impl<'a, 'b, T> From<Position<'a, 'b, T, &'a str>> for Position<'a, 'b, T, [u8]> {
    fn from(input: Position<'a, 'b, T, &'a str>) -> Self {
        Self {
            line: input.line,
            column: input.column,
            index: input.index,
            input: input.input.as_bytes(),
            extra: input.extra,
        }
    }
}

impl<'a, 'b, T> From<Position<'a, 'b, T, [u8]>> for Position<'a, 'b, T, str> {
    fn from(input: Position<'a, 'b, T, [u8]>) -> Self {
        Self {
            line: input.line,
            column: input.column,
            index: input.index,
            input: std::str::from_utf8(input.input).unwrap(),
            extra: input.extra,
        }
    }
}

impl<'a, 'b, T> From<Position<'a, 'b, T, str>> for &'a str {
    fn from(value: Position<'a, 'b, T, str>) -> Self {
        value.input
    }
}

impl<'a, 'b, I, I2, T> Compare<I2> for Position<'a, 'b, T, I>
where
    I: ?Sized,
    &'a I: Compare<I2>,
{
    fn compare(&self, t: I2) -> CompareResult {
        self.input.compare(t)
    }

    fn compare_no_case(&self, t: I2) -> CompareResult {
        self.input.compare_no_case(t)
    }
}

impl<'a, 'b, T, I> Offset for Position<'a, 'b, T, I>
where
    I: ?Sized,
    &'a I: Offset + 'a,
{
    fn offset(&self, second: &Self) -> usize {
        self.input.offset(&second.input)
    }
}

impl<T: FromStr, E> ParseTo<T> for Position<'_, '_, E> {
    fn parse_to(&self) -> Option<T> {
        self.input.parse_to()
    }
}

impl<E> AsBytes for Position<'_, '_, E> {
    fn as_bytes(&self) -> &[u8] {
        self.input.as_bytes()
    }
}

impl<E> Borrow<str> for Position<'_, '_, E, str> {
    fn borrow(&self) -> &str {
        self.input
    }
}

impl<'a, 'b, E> FindSubstring<&'a str> for Position<'a, 'b, E, str> {
    fn find_substring(&self, substr: &'a str) -> Option<usize> {
        self.input.find(substr)
    }
}

impl<'a, 'b> Position<'a, 'b> {
    pub fn new(input: &'a str) -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
            input,
            extra: &(),
        }
    }
}

impl<'a, 'b, T, I: ?Sized> Position<'a, 'b, T, I>
where
    &'a I: AsChars,
{
    /// Splits the input into two positions
    fn left_right_split(&self, left: &'a I, right: &'a I, len: usize) -> (Self, Self) {
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

    pub fn new_with_extra(input: &'a I, extra: &'b T) -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
            input,
            extra,
        }
    }
}

pub type PosResult<'a, 'b, O, T = (), I = str, E = nom::error::Error<Position<'a, 'b, T, I>>> =
    Result<(Position<'a, 'b, T, I>, O), nom::Err<E>>;
