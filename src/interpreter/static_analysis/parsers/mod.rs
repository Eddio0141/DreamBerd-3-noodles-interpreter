//! Contains utilities for parsing the source code into an AST.

#[cfg(test)]
mod tests;

use nom::bytes::complete::*;

use crate::interpreter::parsers::*;

static STRING_QUOTE: [char; 2] = ['"', '\''];

/// Counts the number of newline progress
pub fn ws_progress_count(code: &str) -> usize {
    code.chars().filter(|c| *c == '\n').count()
}

/// Trims starting whitespace
/// # Returns
/// - The rest of the code
/// - Number of lines skipped
pub fn eat_whitespace(code: &str) -> (&str, usize) {
    let ws = |ch| WHITESPACE.contains(&ch);
    let (code, ws) = take_while::<_, _, nom::error::Error<_>>(ws)(code).unwrap();
    let ws = ws_progress_count(ws);
    (code, ws)
}

/// Returns a chunk of something before the next whitespace, then also trims the whitespace
/// # Returns
/// - Chunk slice
/// - The rest of the code
/// - Number of lines skipped from the chunk and whitespace
pub fn eat_chunk(mut code: &str) -> (Option<&str>, &str, usize) {
    if let Some(index) = code.find(&WHITESPACE) {
        let chunk = &code[..index];
        code = &code[index..];
        let (code, skipped) = eat_whitespace(code);
        (Some(chunk), code, skipped + ws_progress_count(chunk))
    } else {
        (Some(code), "", 0)
    }
}

/// Peeks into next chunk
/// # Returns
/// - Chunk slice. If the chunk is all whitespace, then `None` is returned
/// - Number of lines in the chunk
pub fn peek_chunk(code: &str) -> (Option<&str>, usize) {
    if let Some(index) = code.find(&WHITESPACE) {
        let chunk = &code[..index];
        (Some(chunk), ws_progress_count(chunk))
    } else if code.chars().all(|c| WHITESPACE.contains(&c)) {
        (None, 0)
    } else {
        (Some(code), ws_progress_count(code))
    }
}

/// Checks if the chunk of code is a function header
pub fn is_function_header(mut chunk: &str) -> bool {
    // header needs any of the words in order from "function"
    let function = &['f', 'u', 'n', 'c', 't', 'i', 'o', 'n'];
    let Some(start_index) = chunk.find(function) else {
        return false;
    };

    chunk = &chunk[1..];

    // check if in order
    let mut chars_left = &function[start_index..];
    for c in chunk.chars() {
        let c_found = chars_left.iter().position(|l| *l == c);
        match c_found {
            Some(c_found) => {
                chars_left = &chars_left[c_found + 1..];
            }
            None => {
                // not found, so not a function header
                return false;
            }
        }
    }

    true
}

/// Eats chunks until terminator symbol
/// # Returns
/// - Chunk slice containing the terminator symbol and the rest of the code
/// - Number of lines skipped, including after the chunk
///
/// # Note
/// - This doesn't blindly eat chunks until `!` but properly checks string quote balance
pub fn eat_chunks_until_term_in_chunk(code: &str) -> (Option<&str>, &str, usize) {
    // string has 3 stages
    // 0. None - currently not in a string, the chunks are some code
    // 1. Open quote - consumes as much quotes as possible, which is later used to match the close quotes
    // 2. In string - consumes as much as possible until the close quote is matched

    enum StringState<'a> {
        None,
        InString(&'a str),
    }

    let mut string_state = StringState::None;
    let mut total_skipped = 0;

    while let (Some(mut chunk), code, ws_skip) = eat_chunk(code) {
        total_skipped += ws_skip;

        // check and operate depending on if we are in a string
        match string_state {
            StringState::None => {
                // is this a string?
                if chunk.starts_with(&STRING_QUOTE) {
                    // get the quotes
                    let quotes = match chunk.find(&STRING_QUOTE) {
                        Some(index) => &chunk[..index],
                        None => chunk,
                    };

                    string_state = StringState::InString(quotes);
                    continue;
                }
            }
            StringState::InString(quotes) => {
                // is this the end of the string?
                let quote_index = chunk.find(quotes);
                match quote_index {
                    Some(quote_index) => chunk = &chunk[..quote_index],
                    None => continue,
                }
                string_state = StringState::None;
            }
        }

        // see if it contains a terminator
        if chunk.contains('!') {
            return (Some(chunk), code, total_skipped);
        }
    }

    (None, code, total_skipped)
}
