use std::fmt;

use naga::{
    WithSpan, back, front,
    valid::{self, Capabilities, ValidationFlags},
};

#[derive(Debug)]
pub(super) enum Error {
    Parse(front::wgsl::ParseError),
    Validate(Box<WithSpan<valid::ValidationError>>),
    Write(back::wgsl::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "WGSL parse error: {error}"),
            Self::Validate(error) => write!(f, "WGSL validation error: {error}"),
            Self::Write(error) => write!(f, "WGSL write error: {error}"),
        }
    }
}

impl std::error::Error for Error {}

pub(super) fn minify_wgsl(input: &str) -> Result<String, Error> {
    let module = front::wgsl::parse_str(input).map_err(Error::Parse)?;

    let info = valid::Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .map_err(|err| Error::Validate(Box::new(err)))?;

    let output = back::wgsl::write_string(&module, &info, back::wgsl::WriterFlags::empty())
        .map_err(Error::Write)?;

    Ok(minify_whitespace(&output))
}

fn minify_whitespace(input: &str) -> String {
    let mut output = String::with_capacity(input.len());

    let bytes = input.as_bytes();

    let mut i = 0;
    let mut previous_non_space = None;

    while i < bytes.len() {
        let byte = bytes[i];

        if byte.is_ascii_whitespace() {
            i += 1;

            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            if let Some(next) = bytes.get(i).copied()
                && let Some(previous) = previous_non_space
                && needs_separator(previous, next)
            {
                output.push(' ');
            }

            continue;
        }

        output.push(byte as char);
        previous_non_space = Some(byte);
        i += 1;
    }

    output
}

#[inline]
fn needs_separator(previous: u8, next: u8) -> bool {
    is_word_byte(previous) && is_word_byte(next)
}

#[inline]
fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[inline]
fn is_word_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

#[inline]
fn is_word_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[inline]
fn needs_space(previous: TokenKind, current: TokenKind) -> bool {
    matches!((previous, current), (TokenKind::Word, TokenKind::Word))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    None,
    Word,
    String,
    Punctuation,
}
