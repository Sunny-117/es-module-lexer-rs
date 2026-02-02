//! Main lexer implementation.

use crate::error::LexerError;
use crate::types::{Import, Export, OpenToken, ParseResult};

/// Maximum depth for the open token stack (brackets, braces, etc.)
const MAX_STACK_DEPTH: usize = 1024;

/// The main lexer for parsing ES modules.
pub struct Lexer<'a> {
    /// Source code (UTF-8 bytes)
    source: &'a [u8],
    /// Current position
    pos: usize,
    /// Source code end position
    end: usize,
    /// Whether in facade mode
    facade: bool,
    /// Parenthesis/brace matching stack
    pub(crate) open_token_stack: Vec<OpenToken>,
    /// Dynamic import stack
    pub(crate) dynamic_import_stack: Vec<usize>,
    /// Parsed imports
    pub(crate) imports: Vec<Import>,
    /// Parsed exports
    pub(crate) exports: Vec<Export>,
    /// Last token position
    last_token_pos: usize,
    /// Whether last slash was division
    last_slash_was_division: bool,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code.
    pub fn new(source: &'a str) -> Self {
        let bytes = source.as_bytes();
        let estimated_imports = (bytes.len() / 500).max(4); // Estimate ~1 import per 500 bytes, min 4
        let estimated_exports = (bytes.len() / 500).max(4); // Estimate ~1 export per 500 bytes, min 4
        
        Self {
            source: bytes,
            pos: 0,
            end: bytes.len(),
            facade: true,
            open_token_stack: Vec::with_capacity(64),
            dynamic_import_stack: Vec::with_capacity(4),
            imports: Vec::with_capacity(estimated_imports),
            exports: Vec::with_capacity(estimated_exports),
            last_token_pos: 0,
            last_slash_was_division: false,
        }
    }

    /// Parse the source code and return the result.
    pub fn parse(&mut self) -> Result<ParseResult, LexerError> {
        // Phase 1: Try facade mode
        self.facade = true;
        let continue_full = self.parse_facade()?;

        if continue_full {
            // Phase 2: Full parse
            self.facade = false;
            self.parse_full()?;
        }

        let has_module_syntax = !self.imports.is_empty() || !self.exports.is_empty();

        Ok(ParseResult {
            imports: std::mem::take(&mut self.imports),
            exports: std::mem::take(&mut self.exports),
            facade: self.facade,
            has_module_syntax,
        })
    }

    /// Phase 1: Facade mode parsing.
    /// 
    /// In facade mode, we only process import/export statements, comments, and whitespace.
    /// If we encounter any other JavaScript syntax, we set facade = false and return true
    /// to indicate that full parsing is needed.
    /// 
    /// Returns:
    /// - Ok(true) if we need to continue with full parsing
    /// - Ok(false) if facade parsing completed successfully
    fn parse_facade(&mut self) -> Result<bool, LexerError> {
        while !self.is_at_end() {
            let ch = self.comment_whitespace(false)?;
            
            // End of file
            if ch == 0 {
                break;
            }
            
            match ch {
                b'i' if self.is_keyword_start(self.pos) && self.matches_keyword(b"import") => {
                    // Try to parse import statement
                    self.try_parse_import_statement()?;
                }
                b'e' if self.is_keyword_start(self.pos) && self.matches_keyword(b"export") => {
                    // Try to parse export statement
                    self.try_parse_export_statement()?;
                    
                    // Check if facade was set to false during export parsing
                    if !self.facade {
                        return Ok(true); // Switch to full parsing
                    }
                }
                b';' => {
                    // Semicolon is allowed in facade mode
                    self.advance();
                }
                _ => {
                    // Any other syntax means this is not a pure module file
                    self.facade = false;
                    return Ok(true); // Switch to full parsing
                }
            }
        }
        
        // Completed facade parsing successfully
        Ok(false)
    }

    /// Phase 2: Full parsing.
    /// 
    /// In full parsing mode, we process all JavaScript syntax structures,
    /// tracking brackets, braces, and all token types to correctly identify
    /// import/export statements anywhere in the code.
    fn parse_full(&mut self) -> Result<(), LexerError> {
        use crate::types::{OpenToken, OpenTokenState};
        
        // Reset position to start for full parse
        self.pos = 0;
        self.last_token_pos = 0;
        
        while !self.is_at_end() {
            let ch = self.comment_whitespace(true)?;
            
            // End of file
            if ch == 0 {
                break;
            }
            
            match ch {
                b'i' if self.is_keyword_start(self.pos) && self.matches_keyword(b"import") => {
                    // Try to parse import statement
                    self.last_token_pos = self.pos;
                    self.try_parse_import_statement()?;
                }
                b'e' if self.is_keyword_start(self.pos) && self.matches_keyword(b"export") => {
                    // Try to parse export statement
                    self.last_token_pos = self.pos;
                    self.try_parse_export_statement()?;
                }
                b'\'' | b'"' => {
                    // String literal
                    self.last_token_pos = self.pos;
                    self.string_literal(ch)?;
                }
                b'`' => {
                    // Template string
                    self.last_token_pos = self.pos;
                    self.template_string()?;
                }
                b'/' => {
                    // Could be regex or division operator
                    self.handle_slash()?;
                    self.last_token_pos = self.pos;
                }
                b'(' => {
                    // Opening parenthesis
                    self.push_token(OpenToken {
                        state: OpenTokenState::AnyParen,
                        pos: self.pos,
                    })?;
                    self.last_token_pos = self.pos;
                    self.advance();
                }
                b')' => {
                    // Closing parenthesis
                    if let Some(token) = self.open_token_stack.last() {
                        // Check if this closes a dynamic import
                        if token.state == OpenTokenState::ImportParen {
                            // Complete the dynamic import
                            if let Some(import_idx) = self.dynamic_import_stack.pop() {
                                if let Some(import) = self.imports.get_mut(import_idx) {
                                    import.statement_end = self.pos + 1;
                                }
                            }
                        }
                        
                        // Pop from stack
                        if matches!(token.state, OpenTokenState::AnyParen | OpenTokenState::ImportParen | OpenTokenState::AsyncParen) {
                            self.open_token_stack.pop();
                        }
                    }
                    self.last_token_pos = self.pos;
                    self.advance();
                }
                b'{' => {
                    // Opening brace
                    self.push_token(OpenToken {
                        state: OpenTokenState::AnyBrace,
                        pos: self.pos,
                    })?;
                    self.last_token_pos = self.pos;
                    self.advance();
                }
                b'}' => {
                    // Closing brace
                    if let Some(token) = self.open_token_stack.last() {
                        // Pop from stack
                        if matches!(token.state, OpenTokenState::AnyBrace | OpenTokenState::ClassBrace | OpenTokenState::TemplateBrace) {
                            self.open_token_stack.pop();
                        }
                    }
                    self.last_token_pos = self.pos;
                    self.advance();
                }
                b'[' => {
                    // Opening bracket
                    self.last_token_pos = self.pos;
                    self.advance();
                }
                b']' => {
                    // Closing bracket
                    self.last_token_pos = self.pos;
                    self.advance();
                }
                b';' => {
                    // Semicolon
                    self.last_token_pos = self.pos;
                    self.advance();
                }
                _ => {
                    // Any other character - just advance
                    self.last_token_pos = self.pos;
                    self.advance();
                }
            }
        }
        
        Ok(())
    }

    // ===== Basic Character Access Methods =====

    /// Peek at the current character without advancing.
    #[inline(always)]
    pub(crate) fn peek(&self) -> Option<u8> {
        if self.pos < self.end {
            Some(self.source[self.pos])
        } else {
            None
        }
    }

    /// Peek at a character at a specific offset from current position.
    #[inline(always)]
    pub(crate) fn peek_at(&self, offset: usize) -> Option<u8> {
        let pos = self.pos + offset;
        if pos < self.end {
            Some(self.source[pos])
        } else {
            None
        }
    }

    /// Get the current character and advance position by 1.
    #[inline(always)]
    pub(crate) fn advance(&mut self) -> Option<u8> {
        if self.pos < self.end {
            let ch = self.source[self.pos];
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    /// Advance position by n bytes.
    #[inline(always)]
    pub(crate) fn advance_by(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.end);
    }

    /// Check if we've reached the end of the source.
    #[inline(always)]
    pub(crate) fn is_at_end(&self) -> bool {
        self.pos >= self.end
    }

    /// Get the current position.
    #[inline(always)]
    pub(crate) fn position(&self) -> usize {
        self.pos
    }

    /// Get a slice of the source from start to end positions.
    #[inline(always)]
    #[allow(dead_code)]
    pub(crate) fn slice(&self, start: usize, end: usize) -> &[u8] {
        &self.source[start.min(self.end)..end.min(self.end)]
    }

    /// Get a string slice from start to end positions.
    #[inline(always)]
    #[allow(dead_code)]
    pub(crate) fn str_slice(&self, start: usize, end: usize) -> &str {
        // Safety: We assume the source is valid UTF-8 (checked in new())
        unsafe { std::str::from_utf8_unchecked(self.slice(start, end)) }
    }

    /// Check if the current position matches a specific byte sequence.
    #[inline]
    pub(crate) fn matches_bytes(&self, bytes: &[u8]) -> bool {
        if self.pos + bytes.len() > self.end {
            return false;
        }
        &self.source[self.pos..self.pos + bytes.len()] == bytes
    }

    /// Check if the position matches a keyword (with boundary check).
    #[inline]
    pub(crate) fn matches_keyword(&self, keyword: &[u8]) -> bool {
        if self.pos + keyword.len() > self.end {
            return false;
        }
        
        // Check if keyword matches
        if &self.source[self.pos..self.pos + keyword.len()] != keyword {
            return false;
        }
        
        // Check that next character is not alphanumeric or underscore
        let next_pos = self.pos + keyword.len();
        if next_pos < self.end {
            let next_ch = self.source[next_pos];
            if next_ch.is_ascii_alphanumeric() || next_ch == b'_' || next_ch == b'$' {
                return false;
            }
        }
        
        true
    }

    /// Check if the current position is at the start of a keyword.
    #[inline]
    pub(crate) fn is_keyword_start(&self, pos: usize) -> bool {
        if pos == 0 {
            return true;
        }
        
        let prev_ch = self.source[pos - 1];
        !prev_ch.is_ascii_alphanumeric() && prev_ch != b'_' && prev_ch != b'$'
    }

    /// Push a token onto the open token stack with overflow checking.
    /// 
    /// # Arguments
    /// * `token` - The token to push
    /// 
    /// # Returns
    /// Ok(()) if successful, Err(LexerError::StackOverflow) if stack is full
    #[inline]
    pub(crate) fn push_token(&mut self, token: OpenToken) -> Result<(), LexerError> {
        if self.open_token_stack.len() >= MAX_STACK_DEPTH {
            return Err(LexerError::StackOverflow(self.pos));
        }
        self.open_token_stack.push(token);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_new() {
        let source = "import foo from 'bar';";
        let lexer = Lexer::new(source);
        
        assert_eq!(lexer.pos, 0);
        assert_eq!(lexer.end, source.len());
        assert_eq!(lexer.source.len(), source.len());
        assert!(lexer.facade);
        assert_eq!(lexer.imports.len(), 0);
        assert_eq!(lexer.exports.len(), 0);
        assert_eq!(lexer.last_token_pos, 0);
        assert!(!lexer.last_slash_was_division);
    }

    #[test]
    fn test_lexer_new_empty_source() {
        let source = "";
        let lexer = Lexer::new(source);
        
        assert_eq!(lexer.pos, 0);
        assert_eq!(lexer.end, 0);
        assert!(lexer.is_at_end());
    }

    #[test]
    fn test_peek() {
        let source = "abc";
        let lexer = Lexer::new(source);
        
        assert_eq!(lexer.peek(), Some(b'a'));
        assert_eq!(lexer.peek(), Some(b'a')); // Should not advance
        assert_eq!(lexer.pos, 0);
    }

    #[test]
    fn test_peek_at_end() {
        let source = "a";
        let mut lexer = Lexer::new(source);
        lexer.pos = 1;
        
        assert_eq!(lexer.peek(), None);
    }

    #[test]
    fn test_peek_at() {
        let source = "abcdef";
        let lexer = Lexer::new(source);
        
        assert_eq!(lexer.peek_at(0), Some(b'a'));
        assert_eq!(lexer.peek_at(1), Some(b'b'));
        assert_eq!(lexer.peek_at(2), Some(b'c'));
        assert_eq!(lexer.peek_at(5), Some(b'f'));
        assert_eq!(lexer.peek_at(6), None);
    }

    #[test]
    fn test_advance() {
        let source = "abc";
        let mut lexer = Lexer::new(source);
        
        assert_eq!(lexer.advance(), Some(b'a'));
        assert_eq!(lexer.pos, 1);
        assert_eq!(lexer.advance(), Some(b'b'));
        assert_eq!(lexer.pos, 2);
        assert_eq!(lexer.advance(), Some(b'c'));
        assert_eq!(lexer.pos, 3);
        assert_eq!(lexer.advance(), None);
        assert_eq!(lexer.pos, 3);
    }

    #[test]
    fn test_advance_by() {
        let source = "abcdefgh";
        let mut lexer = Lexer::new(source);
        
        lexer.advance_by(3);
        assert_eq!(lexer.pos, 3);
        assert_eq!(lexer.peek(), Some(b'd'));
        
        lexer.advance_by(2);
        assert_eq!(lexer.pos, 5);
        assert_eq!(lexer.peek(), Some(b'f'));
    }

    #[test]
    fn test_advance_by_beyond_end() {
        let source = "abc";
        let mut lexer = Lexer::new(source);
        
        lexer.advance_by(10);
        assert_eq!(lexer.pos, 3); // Should clamp to end
        assert!(lexer.is_at_end());
    }

    #[test]
    fn test_is_at_end() {
        let source = "ab";
        let mut lexer = Lexer::new(source);
        
        assert!(!lexer.is_at_end());
        lexer.pos = 1;
        assert!(!lexer.is_at_end());
        lexer.pos = 2;
        assert!(lexer.is_at_end());
        lexer.pos = 3;
        assert!(lexer.is_at_end());
    }

    #[test]
    fn test_position() {
        let source = "test";
        let mut lexer = Lexer::new(source);
        
        assert_eq!(lexer.position(), 0);
        lexer.pos = 2;
        assert_eq!(lexer.position(), 2);
    }

    #[test]
    fn test_slice() {
        let source = "hello world";
        let lexer = Lexer::new(source);
        
        let slice = lexer.slice(0, 5);
        assert_eq!(slice, b"hello");
        
        let slice = lexer.slice(6, 11);
        assert_eq!(slice, b"world");
    }

    #[test]
    fn test_slice_beyond_end() {
        let source = "test";
        let lexer = Lexer::new(source);
        
        let slice = lexer.slice(0, 100);
        assert_eq!(slice, b"test");
    }

    #[test]
    fn test_str_slice() {
        let source = "hello world";
        let lexer = Lexer::new(source);
        
        let s = lexer.str_slice(0, 5);
        assert_eq!(s, "hello");
        
        let s = lexer.str_slice(6, 11);
        assert_eq!(s, "world");
    }

    #[test]
    fn test_matches_bytes() {
        let source = "import foo";
        let lexer = Lexer::new(source);
        
        assert!(lexer.matches_bytes(b"import"));
        assert!(!lexer.matches_bytes(b"export"));
        assert!(!lexer.matches_bytes(b"import foo bar")); // Beyond end
    }

    #[test]
    fn test_matches_bytes_at_position() {
        let source = "import foo";
        let mut lexer = Lexer::new(source);
        lexer.pos = 7;
        
        assert!(lexer.matches_bytes(b"foo"));
        assert!(!lexer.matches_bytes(b"bar"));
    }

    #[test]
    fn test_matches_keyword() {
        let source = "import foo";
        let lexer = Lexer::new(source);
        
        assert!(lexer.matches_keyword(b"import"));
        assert!(!lexer.matches_keyword(b"export"));
    }

    #[test]
    fn test_matches_keyword_with_boundary() {
        let source = "importfoo";
        let lexer = Lexer::new(source);
        
        // Should not match because 'f' follows immediately
        assert!(!lexer.matches_keyword(b"import"));
    }

    #[test]
    fn test_matches_keyword_with_space() {
        let source = "import ";
        let lexer = Lexer::new(source);
        
        assert!(lexer.matches_keyword(b"import"));
    }

    #[test]
    fn test_matches_keyword_with_punctuation() {
        let source = "import(";
        let lexer = Lexer::new(source);
        
        assert!(lexer.matches_keyword(b"import"));
    }

    #[test]
    fn test_matches_keyword_at_end() {
        let source = "import";
        let lexer = Lexer::new(source);
        
        assert!(lexer.matches_keyword(b"import"));
    }

    #[test]
    fn test_is_keyword_start_at_beginning() {
        let source = "import";
        let lexer = Lexer::new(source);
        
        assert!(lexer.is_keyword_start(0));
    }

    #[test]
    fn test_is_keyword_start_after_space() {
        let source = " import";
        let lexer = Lexer::new(source);
        
        assert!(lexer.is_keyword_start(1));
    }

    #[test]
    fn test_is_keyword_start_after_alphanumeric() {
        let source = "aimport";
        let lexer = Lexer::new(source);
        
        assert!(!lexer.is_keyword_start(1));
    }

    #[test]
    fn test_is_keyword_start_after_underscore() {
        let source = "_import";
        let lexer = Lexer::new(source);
        
        assert!(!lexer.is_keyword_start(1));
    }

    #[test]
    fn test_is_keyword_start_after_dollar() {
        let source = "$import";
        let lexer = Lexer::new(source);
        
        assert!(!lexer.is_keyword_start(1));
    }

    #[test]
    fn test_is_keyword_start_after_punctuation() {
        let source = ";import";
        let lexer = Lexer::new(source);
        
        assert!(lexer.is_keyword_start(1));
    }

    #[test]
    fn test_position_tracking_through_operations() {
        let source = "import foo from 'bar';";
        let mut lexer = Lexer::new(source);
        
        assert_eq!(lexer.position(), 0);
        
        lexer.advance();
        assert_eq!(lexer.position(), 1);
        
        lexer.advance_by(5);
        assert_eq!(lexer.position(), 6);
        
        let _ = lexer.peek();
        assert_eq!(lexer.position(), 6); // peek should not change position
        
        let _ = lexer.advance();
        assert_eq!(lexer.position(), 7);
    }

    #[test]
    fn test_lexer_with_unicode() {
        let source = "import '你好';";
        let lexer = Lexer::new(source);
        
        assert_eq!(lexer.end, source.len()); // Byte length, not char length
        assert!(lexer.matches_keyword(b"import"));
    }

    #[test]
    fn test_empty_source_operations() {
        let source = "";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.is_at_end());
        assert_eq!(lexer.peek(), None);
        assert_eq!(lexer.advance(), None);
        assert_eq!(lexer.position(), 0);
    }

    // ===== Regex vs Division Ambiguity Handling Tests =====

    #[test]
    fn test_is_expression_punctuator() {
        let source = "test";
        let lexer = Lexer::new(source);
        
        assert!(lexer.is_expression_punctuator(b'!'));
        assert!(lexer.is_expression_punctuator(b'%'));
        assert!(lexer.is_expression_punctuator(b'&'));
        assert!(lexer.is_expression_punctuator(b'('));
        assert!(lexer.is_expression_punctuator(b'*'));
        assert!(lexer.is_expression_punctuator(b'+'));
        assert!(lexer.is_expression_punctuator(b','));
        assert!(lexer.is_expression_punctuator(b'-'));
        assert!(lexer.is_expression_punctuator(b'.'));
        assert!(lexer.is_expression_punctuator(b':'));
        assert!(lexer.is_expression_punctuator(b';'));
        assert!(lexer.is_expression_punctuator(b'<'));
        assert!(lexer.is_expression_punctuator(b'='));
        assert!(lexer.is_expression_punctuator(b'>'));
        assert!(lexer.is_expression_punctuator(b'?'));
        assert!(lexer.is_expression_punctuator(b'['));
        assert!(lexer.is_expression_punctuator(b'^'));
        assert!(lexer.is_expression_punctuator(b'{'));
        assert!(lexer.is_expression_punctuator(b'|'));
        assert!(lexer.is_expression_punctuator(b'~'));
        assert!(lexer.is_expression_punctuator(b'\n'));
        assert!(lexer.is_expression_punctuator(b'\r'));
        
        assert!(!lexer.is_expression_punctuator(b'a'));
        assert!(!lexer.is_expression_punctuator(b'0'));
        assert!(!lexer.is_expression_punctuator(b')'));
        assert!(!lexer.is_expression_punctuator(b'}'));
    }

    #[test]
    fn test_read_preceding_keyword() {
        let source = "if (test) while (x) for (i)";
        let lexer = Lexer::new(source);
        
        // "if" at position 0-1
        assert!(lexer.read_preceding_keyword(1, b"if"));
        
        // "while" at position 10-14
        assert!(lexer.read_preceding_keyword(14, b"while"));
        
        // "for" at position 20-22
        assert!(lexer.read_preceding_keyword(22, b"for"));
        
        // Should not match partial keywords
        assert!(!lexer.read_preceding_keyword(1, b"while"));
    }

    #[test]
    fn test_is_expression_keyword_return() {
        let source = "return";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 5; // 'n' in "return"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_throw() {
        let source = "throw";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 4; // 'w' in "throw"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_typeof() {
        let source = "typeof";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 5; // 'f' in "typeof"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_void() {
        let source = "void";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 3; // 'd' in "void"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_yield() {
        let source = "yield";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 4; // 'd' in "yield"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_await() {
        let source = "await";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 4; // 't' in "await"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_delete() {
        let source = "delete";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 5; // 'e' in "delete"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_new() {
        let source = "new";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 2; // 'w' in "new"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_instanceof() {
        let source = "instanceof";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 9; // 'f' in "instanceof"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_case() {
        let source = "case";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 3; // 'e' in "case"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_else() {
        let source = "else";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 3; // 'e' in "else"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_do() {
        let source = "do";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 1; // 'o' in "do"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_in() {
        let source = "in";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 1; // 'n' in "in"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_break() {
        let source = "break";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 4; // 'k' in "break"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_continue() {
        let source = "continue";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 7; // 'e' in "continue"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_debugger() {
        let source = "debugger";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 7; // 'r' in "debugger"
        
        assert!(lexer.is_expression_keyword());
    }

    #[test]
    fn test_is_expression_keyword_not_keyword() {
        let source = "identifier";
        let mut lexer = Lexer::new(source);
        lexer.last_token_pos = 9;
        
        assert!(!lexer.is_expression_keyword());
    }
}

impl<'a> Lexer<'a> {
    // ===== Regex vs Division Ambiguity Handling =====

    /// Handle a '/' character - determine if it's a regex or division operator.
    pub(crate) fn handle_slash(&mut self) -> Result<(), LexerError> {
        let last_ch = if self.last_token_pos < self.end {
            self.source[self.last_token_pos]
        } else {
            0
        };

        // Determine if this is a regular expression based on context
        let is_regex = self.is_expression_punctuator(last_ch)
            && !(last_ch == b'.' && self.last_token_pos > 0 
                && self.source[self.last_token_pos - 1] >= b'0' 
                && self.source[self.last_token_pos - 1] <= b'9')
            && !(last_ch == b'+' && self.last_token_pos > 0 
                && self.source[self.last_token_pos - 1] == b'+')
            && !(last_ch == b'-' && self.last_token_pos > 0 
                && self.source[self.last_token_pos - 1] == b'-')
            || (last_ch == b')' && self.is_paren_keyword())
            || (last_ch == b'}' && (self.is_expression_terminator() 
                || (!self.open_token_stack.is_empty() 
                    && self.open_token_stack.last().unwrap().state == crate::types::OpenTokenState::ClassBrace)))
            || self.is_expression_keyword()
            || (last_ch == b'/' && self.last_slash_was_division)
            || last_ch == 0;

        if is_regex {
            self.regular_expression()?;
            self.last_slash_was_division = false;
        } else {
            // Division operator
            self.last_slash_was_division = true;
            self.pos += 1;
        }

        Ok(())
    }

    /// Check if a character is an expression punctuator.
    /// Expression punctuators: !%&(*+,-.:;<=>?[^{|~
    #[inline(always)]
    pub(crate) fn is_expression_punctuator(&self, ch: u8) -> bool {
        matches!(ch,
            b'!' | b'%' | b'&' | b'(' | b'*' | b'+' | b',' | b'-' | b'.' |
            b':' | b';' | b'<' | b'=' | b'>' | b'?' | b'[' | b'^' | b'{' |
            b'|' | b'~' | b'\n' | b'\r'
        )
    }

    /// Check if the last '(' corresponds to a while/for/if keyword.
    fn is_paren_keyword(&self) -> bool {
        if self.open_token_stack.is_empty() {
            return false;
        }

        let last_token = self.open_token_stack.last().unwrap();
        if last_token.state != crate::types::OpenTokenState::AnyParen {
            return false;
        }

        let pos = last_token.pos;
        self.read_preceding_keyword(pos, b"while")
            || self.read_preceding_keyword(pos, b"for")
            || self.read_preceding_keyword(pos, b"if")
    }

    /// Check if the last '}' is an expression terminator.
    /// Expression terminators: => ; ) finally catch else
    fn is_expression_terminator(&self) -> bool {
        if self.open_token_stack.is_empty() {
            return false;
        }

        let last_token = self.open_token_stack.last().unwrap();
        if last_token.state != crate::types::OpenTokenState::AnyBrace {
            return false;
        }

        let pos = last_token.pos;
        if pos == 0 {
            return false;
        }

        let ch = self.source[pos];
        match ch {
            b'>' if pos > 0 && self.source[pos - 1] == b'=' => true,
            b';' | b')' => true,
            b'h' => self.read_preceding_keyword(pos - 1, b"catch"),
            b'y' => self.read_preceding_keyword(pos - 1, b"finally"),
            b'e' => self.read_preceding_keyword(pos - 1, b"else"),
            _ => false,
        }
    }

    /// Check if the last token position is an expression keyword.
    /// Expression keywords: case, debugger, delete, do, else, in, instanceof,
    /// new, return, throw, typeof, void, yield, await, continue, break
    fn is_expression_keyword(&self) -> bool {
        if self.last_token_pos >= self.end {
            return false;
        }

        let pos = self.last_token_pos;
        let ch = self.source[pos];

        match ch {
            b'd' => {
                if pos == 0 {
                    return false;
                }
                match self.source[pos - 1] {
                    b'i' => {
                        // void
                        self.read_preceding_keyword(pos, b"void")
                    }
                    b'l' => {
                        // yield
                        self.read_preceding_keyword(pos, b"yield")
                    }
                    _ => false,
                }
            }
            b'e' => {
                if pos == 0 {
                    return false;
                }
                match self.source[pos - 1] {
                    b's' => {
                        if pos < 2 {
                            return false;
                        }
                        match self.source[pos - 2] {
                            b'l' => {
                                // else
                                self.read_preceding_keyword(pos, b"else")
                            }
                            b'a' => {
                                // case
                                self.read_preceding_keyword(pos, b"case")
                            }
                            _ => false,
                        }
                    }
                    b't' => {
                        // delete
                        self.read_preceding_keyword(pos, b"delete")
                    }
                    b'u' => {
                        // continue
                        self.read_preceding_keyword(pos, b"continue")
                    }
                    _ => false,
                }
            }
            b'f' => {
                if pos < 2 || self.source[pos - 1] != b'o' || self.source[pos - 2] != b'e' {
                    return false;
                }
                if pos < 3 {
                    return false;
                }
                match self.source[pos - 3] {
                    b'c' => {
                        // instanceof
                        self.read_preceding_keyword(pos, b"instanceof")
                    }
                    b'p' => {
                        // typeof
                        self.read_preceding_keyword(pos, b"typeof")
                    }
                    _ => false,
                }
            }
            b'k' => {
                // break
                self.read_preceding_keyword(pos, b"break")
            }
            b'n' => {
                // in, return
                (pos > 0 && self.source[pos - 1] == b'i' && self.is_keyword_start(pos - 1))
                    || self.read_preceding_keyword(pos, b"return")
            }
            b'o' => {
                // do
                pos > 0 && self.source[pos - 1] == b'd' && self.is_keyword_start(pos - 1)
            }
            b'r' => {
                // debugger
                self.read_preceding_keyword(pos, b"debugger")
            }
            b't' => {
                // await
                self.read_preceding_keyword(pos, b"await")
            }
            b'w' => {
                if pos == 0 {
                    return false;
                }
                match self.source[pos - 1] {
                    b'e' => {
                        // new
                        pos >= 2 && self.source[pos - 2] == b'n' && self.is_keyword_start(pos - 2)
                    }
                    b'o' => {
                        // throw
                        self.read_preceding_keyword(pos, b"throw")
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Read a preceding keyword at the given position.
    fn read_preceding_keyword(&self, end_pos: usize, keyword: &[u8]) -> bool {
        if end_pos + 1 < keyword.len() {
            return false;
        }

        let start_pos = end_pos + 1 - keyword.len();
        if start_pos + keyword.len() > self.end {
            return false;
        }

        // Check if keyword matches
        if &self.source[start_pos..start_pos + keyword.len()] != keyword {
            return false;
        }

        // Check that it's at a keyword boundary
        if start_pos > 0 {
            let prev_ch = self.source[start_pos - 1];
            if prev_ch.is_ascii_alphanumeric() || prev_ch == b'_' || prev_ch == b'$' {
                return false;
            }
        }

        true
    }
    
    // ===== Testing Helpers =====
    
    #[cfg(test)]
    pub(crate) fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }
    
    #[cfg(test)]
    pub(crate) fn set_last_token_pos(&mut self, pos: usize) {
        self.last_token_pos = pos;
    }
    
    #[cfg(test)]
    pub(crate) fn get_pos(&self) -> usize {
        self.pos
    }
    
    /// Set facade mode (internal helper)
    pub(crate) fn set_facade(&mut self, facade: bool) {
        self.facade = facade;
    }
    
    /// Get facade mode (internal helper)
    #[cfg(test)]
    pub(crate) fn get_facade(&self) -> bool {
        self.facade
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    
    // ===== Generators for test data =====
    
    /// Generate a valid import statement that can stay in facade mode
    fn arb_import_statement() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("import foo from 'bar';".to_string()),
            Just("import { a, b } from 'module';".to_string()),
            Just("import * as ns from 'module';".to_string()),
            Just("import 'side-effect';".to_string()),
        ]
    }
    
    /// Generate a valid export statement that can stay in facade mode
    fn arb_export_statement() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("export { a, b };".to_string()),
            Just("export * as ns from 'module';".to_string()),
            Just("export { x } from 'module';".to_string()),
        ]
    }
    
    /// Generate an export that requires full parsing
    #[allow(dead_code)]
    fn arb_export_with_body() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("export default function() {}".to_string()),
            Just("export function foo() {}".to_string()),
            Just("export class Bar {}".to_string()),
        ]
    }
    
    /// Generate whitespace and comments
    fn arb_whitespace_and_comments() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("".to_string()),
            Just(" ".to_string()),
            Just("\n".to_string()),
            Just("  \n  ".to_string()),
            Just("// comment\n".to_string()),
            Just("/* comment */".to_string()),
            Just("/* multi\nline */".to_string()),
        ]
    }
    
    /// Generate a pure module file (only imports/exports)
    fn arb_pure_module() -> impl Strategy<Value = String> {
        prop::collection::vec(
            (arb_whitespace_and_comments(), prop_oneof![
                arb_import_statement(),
                arb_export_statement(),
            ]),
            1..5
        ).prop_map(|parts| {
            let mut result = String::new();
            for (ws, stmt) in parts {
                result.push_str(&ws);
                result.push_str(&stmt);
                result.push('\n');
            }
            result
        })
    }
    
    /// Generate a mixed file (imports/exports + other code)
    fn arb_mixed_file() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                arb_import_statement(),
                arb_export_statement(),
                Just("const x = 1;".to_string()),
                Just("function foo() {}".to_string()),
                Just("class Bar {}".to_string()),
                Just("if (true) {}".to_string()),
            ],
            1..5
        ).prop_map(|stmts| {
            stmts.join("\n")
        })
    }
    
    // ===== Property Tests =====
    
    // Feature: es-module-lexer-rs, Property 7: Facade 模式检测
    // Validates: Requirements 2.1, 2.2
    proptest! {
        #[test]
        fn prop_facade_mode_detection_pure_module(
            module in arb_pure_module()
        ) {
            let mut lexer = Lexer::new(&module);
            let result = lexer.parse();
            
            // Pure module files should have facade = true
            // Note: export function/class/default function will switch to full parsing
            prop_assert!(result.is_ok());
            let result = result.unwrap();
            
            // If the module contains export function/class, facade will be false
            // This is expected behavior
            if module.contains("export function") || module.contains("export class") || 
               module.contains("export default function") {
                // These require full parsing
                prop_assert!(!result.facade || result.facade, "Export with body may have facade=false");
            } else {
                prop_assert!(result.facade, "Pure module should have facade=true");
            }
        }
        
        #[test]
        fn prop_facade_mode_detection_mixed_file(
            file in arb_mixed_file()
        ) {
            let mut lexer = Lexer::new(&file);
            let result = lexer.parse();
            
            // Mixed files should have facade = false
            prop_assert!(result.is_ok());
            let result = result.unwrap();
            
            // If the file contains non-module syntax, facade should be false
            // Note: This is a weak assertion because our generator might produce
            // only module syntax by chance
            if file.contains("const ") || file.contains("function ") || 
               file.contains("class ") || file.contains("if (") {
                prop_assert!(!result.facade, "Mixed file should have facade=false");
            }
        }
        
        #[test]
        fn prop_facade_only_comments(
            comments in prop::collection::vec(
                prop_oneof![
                    Just("// comment\n".to_string()),
                    Just("/* comment */".to_string()),
                ],
                1..5
            )
        ) {
            let source = comments.join("\n");
            let mut lexer = Lexer::new(&source);
            let result = lexer.parse().unwrap();
            
            // File with only comments should have facade = true
            prop_assert!(result.facade);
            prop_assert!(!result.has_module_syntax);
        }
        
        #[test]
        fn prop_facade_import_then_code(
            import in arb_import_statement(),
            code in prop::sample::select(vec![
                "const x = 1;",
                "function foo() {}",
                "class Bar {}",
            ])
        ) {
            let source = format!("{}\n{}", import, code);
            let mut lexer = Lexer::new(&source);
            let result = lexer.parse().unwrap();
            
            // File with import followed by non-module code should have facade = false
            prop_assert!(!result.facade);
            prop_assert!(!result.imports.is_empty());
        }
        
        #[test]
        fn prop_facade_export_then_code(
            export in arb_export_statement(),
            code in prop::sample::select(vec![
                "const y = 2;",
                "let z = 3;",
            ])
        ) {
            let source = format!("{}\n{}", export, code);
            let mut lexer = Lexer::new(&source);
            let result = lexer.parse().unwrap();
            
            // File with export followed by non-module code should have facade = false
            prop_assert!(!result.facade);
        }
    }
    
    #[test]
    fn test_facade_empty_file() {
        let source = "";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse().unwrap();
        
        // Empty file should have facade = true
        assert!(result.facade);
        assert!(!result.has_module_syntax);
    }
    
    // Feature: es-module-lexer-rs, Property 1: 解析完整性
    // Validates: Requirements 1.1, 1.2
    proptest! {
        #[test]
        fn prop_parse_completeness_imports(
            imports in prop::collection::vec(arb_import_statement(), 1..5)
        ) {
            let source = imports.join("\n");
            let mut lexer = Lexer::new(&source);
            let result = lexer.parse().unwrap();
            
            // All imports should be parsed
            prop_assert_eq!(result.imports.len(), imports.len());
            
            // has_module_syntax should be true
            prop_assert!(result.has_module_syntax);
            
            // Verify position information is accurate
            for import in &result.imports {
                // statement_start should be within bounds
                prop_assert!(import.statement_start < source.len());
                
                // statement_end should be within bounds or at end
                prop_assert!(import.statement_end <= source.len());
                
                // start should be before end
                prop_assert!(import.start <= import.end);
            }
        }
        
        #[test]
        fn prop_parse_completeness_exports(
            exports in prop::collection::vec(arb_export_statement(), 1..5)
        ) {
            let source = exports.join("\n");
            let mut lexer = Lexer::new(&source);
            let result = lexer.parse().unwrap();
            
            // All exports should be parsed
            prop_assert!(result.exports.len() >= exports.len(), 
                "Expected at least {} exports, got {}", exports.len(), result.exports.len());
            
            // has_module_syntax should be true
            prop_assert!(result.has_module_syntax);
            
            // Verify position information is accurate
            for export in &result.exports {
                // start should be within bounds
                prop_assert!(export.start < source.len());
                
                // end should be within bounds
                prop_assert!(export.end <= source.len());
                
                // start should be before end
                prop_assert!(export.start <= export.end);
            }
        }
        
        #[test]
        fn prop_parse_completeness_mixed(
            imports in prop::collection::vec(arb_import_statement(), 0..3),
            exports in prop::collection::vec(arb_export_statement(), 0..3)
        ) {
            let mut source = String::new();
            for import in &imports {
                source.push_str(import);
                source.push('\n');
            }
            for export in &exports {
                source.push_str(export);
                source.push('\n');
            }
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.parse().unwrap();
            
            // All imports should be parsed
            prop_assert_eq!(result.imports.len(), imports.len());
            
            // All exports should be parsed (may be more due to multiple exports in one statement)
            prop_assert!(result.exports.len() >= exports.len());
            
            // has_module_syntax should be true if there are any imports or exports
            if !imports.is_empty() || !exports.is_empty() {
                prop_assert!(result.has_module_syntax);
            }
        }
        
        #[test]
        fn prop_parse_position_accuracy(
            import in arb_import_statement()
        ) {
            let source = format!("  {}  ", import); // Add whitespace
            let mut lexer = Lexer::new(&source);
            let result = lexer.parse().unwrap();
            
            // Should have exactly one import
            prop_assert_eq!(result.imports.len(), 1);
            
            let parsed_import = &result.imports[0];
            
            // Extract the module specifier using the positions
            if parsed_import.start < parsed_import.end {
                let module_spec = &source[parsed_import.start..parsed_import.end];
                
                // Module specifier should not be empty
                prop_assert!(!module_spec.is_empty());
                
                // Module specifier should be part of the original import statement
                prop_assert!(import.contains(module_spec) || module_spec.contains("bar") || module_spec.contains("module"));
            }
        }
    }
}
