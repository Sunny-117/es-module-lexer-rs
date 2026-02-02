//! # es-module-lexer
//!
//! Fast lexer for ES module imports and exports.
//!
//! This is a Rust implementation of es-module-lexer, providing the same
//! functionality with improved performance and memory safety.

pub mod error;
pub mod lexer;
pub mod parser;
pub mod scanner;
pub mod types;

#[cfg(test)]
mod error_tests;

pub use error::LexerError;
pub use lexer::Lexer;
pub use types::{Attribute, Export, Import, ImportType, ParseResult};

/// Parse JavaScript source code to extract imports and exports.
///
/// # Examples
///
/// ```
/// use es_module_lexer::parse;
///
/// let source = r#"import foo from 'bar';"#;
/// let result = parse(source).unwrap();
/// // Note: Full implementation will be added in later tasks
/// assert_eq!(result.facade, true);
/// ```
pub fn parse(source: &str) -> Result<ParseResult, LexerError> {
    let mut lexer = Lexer::new(source);
    lexer.parse()
}
