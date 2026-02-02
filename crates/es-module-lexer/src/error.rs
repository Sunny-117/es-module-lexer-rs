//! Error types for the lexer.

use thiserror::Error;

/// Errors that can occur during lexing.
#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unexpected token at position {0}")]
    UnexpectedToken(usize),

    #[error("Unterminated string starting at position {0}")]
    UnterminatedString(usize),

    #[error("Unterminated comment starting at position {0}")]
    UnterminatedComment(usize),

    #[error("Unterminated regular expression starting at position {0}")]
    UnterminatedRegex(usize),

    #[error("Invalid escape sequence at position {0}")]
    InvalidEscape(usize),

    #[error("Expected colon at position {0}")]
    ExpectedColon(usize),

    #[error("Expected string at position {0}")]
    ExpectedString(usize),

    #[error("Stack overflow at position {0}")]
    StackOverflow(usize),

    #[error("Invalid UTF-8 in source code")]
    InvalidUtf8,
}
