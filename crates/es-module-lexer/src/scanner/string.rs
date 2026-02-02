//! String literal scanning.

use crate::lexer::Lexer;
use crate::error::LexerError;

impl<'a> Lexer<'a> {
    /// Scan a string literal (single or double quoted).
    /// 
    /// Handles escape sequences including:
    /// - Simple escapes: \n, \r, \t, \b, \v, \f, \0
    /// - Hex escapes: \xHH
    /// - Unicode escapes: \uHHHH, \u{HHHHHH}
    /// - Line continuations: \<newline>
    /// 
    /// The position should be at the opening quote when called.
    /// After successful parsing, position will be after the closing quote.
    pub(crate) fn string_literal(&mut self, quote: u8) -> Result<(), LexerError> {
        let start = self.position();
        self.advance_by(1); // Skip opening quote
        
        loop {
            if self.is_at_end() {
                return Err(LexerError::UnterminatedString(start));
            }
            
            let ch = self.peek().unwrap();
            
            if ch == quote {
                // String ends
                self.advance_by(1); // Skip closing quote
                return Ok(());
            }
            
            if ch == b'\\' {
                // Escape sequence
                self.advance_by(1);
                
                if self.is_at_end() {
                    return Err(LexerError::UnterminatedString(start));
                }
                
                let escaped = self.peek().unwrap();
                self.advance_by(1);
                
                match escaped {
                    // Simple escapes
                    b'n' | b'r' | b't' | b'b' | b'v' | b'f' | b'0' | b'\\' | b'\'' | b'"' => {
                        // Single character escape, already advanced
                    }
                    b'x' => {
                        // \xHH - 2 hex digits
                        self.read_hex_chars(2)?;
                    }
                    b'u' => {
                        // \uHHHH or \u{HHHHHH}
                        if !self.is_at_end() && self.peek().unwrap() == b'{' {
                            self.advance_by(1); // Skip '{'
                            self.read_hex_until(b'}')?;
                        } else {
                            // \uHHHH - 4 hex digits
                            self.read_hex_chars(4)?;
                        }
                    }
                    b'\r' => {
                        // Line continuation - skip \r and optional \n
                        if !self.is_at_end() && self.peek().unwrap() == b'\n' {
                            self.advance_by(1);
                        }
                    }
                    b'\n' => {
                        // Line continuation - already advanced
                    }
                    _ => {
                        // Other characters are kept as-is (e.g., \a becomes 'a')
                    }
                }
            } else if ch == b'\r' || ch == b'\n' {
                // Unescaped newline in string
                return Err(LexerError::UnterminatedString(start));
            } else {
                // Regular character
                self.advance_by(1);
            }
        }
    }

    /// Read and extract the string value from a string literal.
    /// 
    /// This method decodes escape sequences and returns the actual string value.
    /// The position should be at the opening quote when called.
    /// After successful parsing, position will be after the closing quote.
    #[allow(dead_code)]
    pub(crate) fn read_string(&mut self, quote: u8) -> Result<String, LexerError> {
        let start = self.position();
        self.advance_by(1); // Skip opening quote
        
        let mut result = String::new();
        let mut chunk_start = self.position();
        
        loop {
            if self.is_at_end() {
                return Err(LexerError::UnterminatedString(start));
            }
            
            let ch = self.peek().unwrap();
            let current_pos = self.position();
            
            if ch == quote {
                // String ends - add final chunk
                if chunk_start < current_pos {
                    result.push_str(self.str_slice(chunk_start, current_pos));
                }
                self.advance_by(1); // Skip closing quote
                return Ok(result);
            }
            
            if ch == b'\\' {
                // Add chunk before escape
                if chunk_start < current_pos {
                    result.push_str(self.str_slice(chunk_start, current_pos));
                }
                
                self.advance_by(1);
                
                if self.is_at_end() {
                    return Err(LexerError::UnterminatedString(start));
                }
                
                let escaped = self.peek().unwrap();
                self.advance_by(1);
                
                match escaped {
                    b'n' => result.push('\n'),
                    b'r' => result.push('\r'),
                    b't' => result.push('\t'),
                    b'b' => result.push('\u{0008}'),
                    b'v' => result.push('\u{000B}'),
                    b'f' => result.push('\u{000C}'),
                    b'0' => result.push('\0'),
                    b'\\' => result.push('\\'),
                    b'\'' => result.push('\''),
                    b'"' => result.push('"'),
                    b'x' => {
                        // \xHH
                        let hex = self.read_hex_chars(2)?;
                        result.push(char::from_u32(hex).unwrap_or('\u{FFFD}'));
                    }
                    b'u' => {
                        // \uHHHH or \u{HHHHHH}
                        if !self.is_at_end() && self.peek().unwrap() == b'{' {
                            self.advance_by(1); // Skip '{'
                            let hex = self.read_hex_until(b'}')?;
                            result.push(char::from_u32(hex).unwrap_or('\u{FFFD}'));
                        } else {
                            let hex = self.read_hex_chars(4)?;
                            result.push(char::from_u32(hex).unwrap_or('\u{FFFD}'));
                        }
                    }
                    b'\r' => {
                        // Line continuation - skip \r and optional \n
                        if !self.is_at_end() && self.peek().unwrap() == b'\n' {
                            self.advance_by(1);
                        }
                        // Don't add anything to result
                    }
                    b'\n' => {
                        // Line continuation - don't add anything to result
                    }
                    _ => {
                        // Other escaped characters are kept as-is
                        result.push(escaped as char);
                    }
                }
                
                chunk_start = self.position();
            } else if ch == b'\r' || ch == b'\n' {
                // Unescaped newline in string
                return Err(LexerError::UnterminatedString(start));
            } else {
                // Regular character
                self.advance_by(1);
            }
        }
    }

    /// Read exactly `count` hexadecimal digits and return their value.
    fn read_hex_chars(&mut self, count: usize) -> Result<u32, LexerError> {
        let mut value = 0u32;
        for _ in 0..count {
            if self.is_at_end() {
                return Err(LexerError::InvalidEscape(self.position()));
            }
            let ch = self.peek().unwrap();
            let digit = match ch {
                b'0'..=b'9' => (ch - b'0') as u32,
                b'a'..=b'f' => (ch - b'a' + 10) as u32,
                b'A'..=b'F' => (ch - b'A' + 10) as u32,
                _ => return Err(LexerError::InvalidEscape(self.position())),
            };
            value = value * 16 + digit;
            self.advance_by(1);
        }
        Ok(value)
    }

    /// Read hexadecimal digits until the terminator character is found.
    /// Returns the parsed value and advances past the terminator.
    fn read_hex_until(&mut self, terminator: u8) -> Result<u32, LexerError> {
        let mut value = 0u32;
        let mut digit_count = 0;
        
        loop {
            if self.is_at_end() {
                return Err(LexerError::InvalidEscape(self.position()));
            }
            
            let ch = self.peek().unwrap();
            
            if ch == terminator {
                if digit_count == 0 {
                    return Err(LexerError::InvalidEscape(self.position()));
                }
                self.advance_by(1); // Skip terminator
                return Ok(value);
            }
            
            let digit = match ch {
                b'0'..=b'9' => (ch - b'0') as u32,
                b'a'..=b'f' => (ch - b'a' + 10) as u32,
                b'A'..=b'F' => (ch - b'A' + 10) as u32,
                _ => return Err(LexerError::InvalidEscape(self.position())),
            };
            
            value = value * 16 + digit;
            digit_count += 1;
            self.advance_by(1);
            
            // Unicode code points must be <= 0x10FFFF
            if value > 0x10FFFF {
                return Err(LexerError::InvalidEscape(self.position()));
            }
        }
    }

    /// Scan a template string.
    /// 
    /// Handles template literals with ${} expression interpolation.
    /// Tracks nesting levels using the OpenToken stack.
    /// 
    /// The position should be at the opening backtick when called.
    /// After successful parsing, position will be after the closing backtick.
    pub(crate) fn template_string(&mut self) -> Result<(), LexerError> {
        use crate::types::{OpenToken, OpenTokenState};
        
        let start = self.position();
        self.advance_by(1); // Skip opening backtick
        
        // Push template token onto stack
        self.push_token(OpenToken {
            state: OpenTokenState::Template,
            pos: start,
        })?;
        
        loop {
            if self.is_at_end() {
                return Err(LexerError::UnterminatedString(start));
            }
            
            let ch = self.peek().unwrap();
            
            match ch {
                b'`' => {
                    // Template string ends
                    self.advance_by(1);
                    
                    // Pop template token from stack
                    if let Some(token) = self.open_token_stack.last() {
                        if token.state == OpenTokenState::Template {
                            self.open_token_stack.pop();
                        }
                    }
                    
                    return Ok(());
                }
                b'$' => {
                    // Check for ${}
                    if self.peek_at(1) == Some(b'{') {
                        self.advance_by(2); // Skip ${
                        
                        // Push template brace onto stack
                        self.push_token(OpenToken {
                            state: OpenTokenState::TemplateBrace,
                            pos: self.position() - 1,
                        })?;
                        
                        // Parse the expression inside ${}
                        self.parse_template_expression()?;
                    } else {
                        self.advance_by(1);
                    }
                }
                b'\\' => {
                    // Escape sequence
                    self.advance_by(1);
                    
                    if self.is_at_end() {
                        return Err(LexerError::UnterminatedString(start));
                    }
                    
                    let escaped = self.peek().unwrap();
                    self.advance_by(1);
                    
                    // Handle special escape sequences
                    match escaped {
                        b'x' => {
                            // \xHH
                            self.read_hex_chars(2)?;
                        }
                        b'u' => {
                            // \uHHHH or \u{HHHHHH}
                            if !self.is_at_end() && self.peek().unwrap() == b'{' {
                                self.advance_by(1); // Skip '{'
                                self.read_hex_until(b'}')?;
                            } else {
                                self.read_hex_chars(4)?;
                            }
                        }
                        _ => {
                            // Other escapes are handled as-is
                        }
                    }
                }
                _ => {
                    // Regular character (including newlines, which are allowed in templates)
                    self.advance_by(1);
                }
            }
        }
    }
    
    /// Parse an expression inside ${} in a template string.
    /// Handles nested braces, parentheses, and nested template strings.
    fn parse_template_expression(&mut self) -> Result<(), LexerError> {
        use crate::types::{OpenToken, OpenTokenState};
        
        let mut brace_depth = 1; // We're already inside one ${}
        
        loop {
            if self.is_at_end() {
                return Err(LexerError::UnterminatedString(self.position()));
            }
            
            let ch = self.peek().unwrap();
            
            match ch {
                b'{' => {
                    self.advance_by(1);
                    brace_depth += 1;
                    self.push_token(OpenToken {
                        state: OpenTokenState::AnyBrace,
                        pos: self.position() - 1,
                    })?;
                }
                b'}' => {
                    self.advance_by(1);
                    brace_depth -= 1;
                    
                    // Pop from stack
                    if let Some(token) = self.open_token_stack.last() {
                        if token.state == OpenTokenState::AnyBrace || token.state == OpenTokenState::TemplateBrace {
                            self.open_token_stack.pop();
                        }
                    }
                    
                    if brace_depth == 0 {
                        // End of template expression
                        return Ok(());
                    }
                }
                b'(' => {
                    self.advance_by(1);
                    self.push_token(OpenToken {
                        state: OpenTokenState::AnyParen,
                        pos: self.position() - 1,
                    })?;
                }
                b')' => {
                    self.advance_by(1);
                    if let Some(token) = self.open_token_stack.last() {
                        if token.state == OpenTokenState::AnyParen {
                            self.open_token_stack.pop();
                        }
                    }
                }
                b'[' => {
                    self.advance_by(1);
                }
                b']' => {
                    self.advance_by(1);
                }
                b'\'' | b'"' => {
                    // String literal inside template expression
                    self.string_literal(ch)?;
                }
                b'`' => {
                    // Nested template string
                    self.template_string()?;
                }
                b'/' => {
                    // Could be regex or division - use existing logic
                    if self.peek_at(1) == Some(b'/') {
                        // Line comment
                        self.advance_by(2);
                        while !self.is_at_end() {
                            let ch = self.peek().unwrap();
                            if ch == b'\n' || ch == b'\r' {
                                break;
                            }
                            self.advance_by(1);
                        }
                    } else if self.peek_at(1) == Some(b'*') {
                        // Block comment
                        self.advance_by(2);
                        while !self.is_at_end() {
                            if self.peek() == Some(b'*') && self.peek_at(1) == Some(b'/') {
                                self.advance_by(2);
                                break;
                            }
                            self.advance_by(1);
                        }
                    } else {
                        // Could be regex or division - for simplicity, just advance
                        // In a full implementation, we'd use handle_slash()
                        self.advance_by(1);
                    }
                }
                b'\\' => {
                    // Escape in template expression (shouldn't happen, but handle it)
                    self.advance_by(1);
                    if !self.is_at_end() {
                        self.advance_by(1);
                    }
                }
                _ => {
                    // Regular character
                    self.advance_by(1);
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_single_quote_string() {
        let source = "'hello'";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.string_literal(b'\'').is_ok());
        assert_eq!(lexer.position(), 7);
    }

    #[test]
    fn test_simple_double_quote_string() {
        let source = "\"hello\"";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.string_literal(b'"').is_ok());
        assert_eq!(lexer.position(), 7);
    }

    #[test]
    fn test_empty_string() {
        let source = "''";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.string_literal(b'\'').is_ok());
        assert_eq!(lexer.position(), 2);
    }

    #[test]
    fn test_unterminated_string() {
        let source = "'hello";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.string_literal(b'\'');
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    #[test]
    fn test_string_with_escaped_quote() {
        let source = "'hello\\'world'";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.string_literal(b'\'').is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_string_with_newline_escape() {
        let source = "'hello\\nworld'";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.string_literal(b'\'').is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_string_with_hex_escape() {
        let source = "'\\x41'"; // 'A'
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.string_literal(b'\'').is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_string_with_unicode_escape() {
        let source = "'\\u0041'"; // 'A'
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.string_literal(b'\'').is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_string_with_unicode_brace_escape() {
        let source = "'\\u{1F600}'"; // 😀
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.string_literal(b'\'').is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_string_with_unescaped_newline() {
        let source = "'hello\nworld'";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.string_literal(b'\'');
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    #[test]
    fn test_read_string_simple() {
        let source = "'hello'";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'').unwrap();
        assert_eq!(result, "hello");
        assert_eq!(lexer.position(), 7);
    }

    #[test]
    fn test_read_string_with_escapes() {
        let source = "'hello\\nworld'";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'').unwrap();
        assert_eq!(result, "hello\nworld");
    }

    #[test]
    fn test_read_string_with_hex_escape() {
        let source = "'\\x41BC'"; // 'ABC'
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'').unwrap();
        assert_eq!(result, "ABC");
    }

    #[test]
    fn test_read_string_with_unicode_escape() {
        let source = "'\\u0041BC'"; // 'ABC'
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'').unwrap();
        assert_eq!(result, "ABC");
    }

    #[test]
    fn test_read_string_with_unicode_brace_escape() {
        let source = "'\\u{1F600}'"; // 😀
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'').unwrap();
        assert_eq!(result, "😀");
    }

    #[test]
    fn test_read_string_all_simple_escapes() {
        let source = "'\\n\\r\\t\\b\\v\\f\\0\\\\\\'\\\"'";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'').unwrap();
        assert_eq!(result, "\n\r\t\u{0008}\u{000B}\u{000C}\0\\'\"");
    }

    #[test]
    fn test_read_string_line_continuation() {
        let source = "'hello\\\nworld'";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'').unwrap();
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_read_string_line_continuation_crlf() {
        let source = "'hello\\\r\nworld'";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'').unwrap();
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_invalid_hex_escape() {
        let source = "'\\xGG'";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'');
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::InvalidEscape(_)));
    }

    #[test]
    fn test_invalid_unicode_escape() {
        let source = "'\\uGGGG'";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'');
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::InvalidEscape(_)));
    }

    #[test]
    fn test_incomplete_hex_escape() {
        let source = "'\\x4'";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'');
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_unicode_brace_escape() {
        let source = "'\\u{}'";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'');
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::InvalidEscape(_)));
    }

    #[test]
    fn test_unicode_brace_escape_too_large() {
        let source = "'\\u{110000}'"; // > 0x10FFFF
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_string(b'\'');
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::InvalidEscape(_)));
    }

    // ===== Template String Tests =====

    #[test]
    fn test_simple_template_string() {
        let source = "`hello world`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_empty_template_string() {
        let source = "``";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), 2);
    }

    #[test]
    fn test_template_string_with_newline() {
        let source = "`hello\nworld`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_simple_interpolation() {
        let source = "`hello ${name}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_multiple_interpolations() {
        let source = "`${a} + ${b} = ${c}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_expression() {
        let source = "`result: ${1 + 2}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_nested_braces() {
        let source = "`obj: ${{ a: 1, b: 2 }}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_nested_template() {
        let source = "`outer ${`inner`}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_deeply_nested_template() {
        let source = "`a ${`b ${`c`}`}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_string_in_interpolation() {
        let source = "`hello ${'world'}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_double_quote_string_in_interpolation() {
        let source = "`hello ${\"world\"}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_function_call() {
        let source = "`result: ${func(a, b)}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_array_access() {
        let source = "`item: ${arr[0]}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_escape_sequences() {
        let source = "`hello\\nworld\\t!`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_escaped_backtick() {
        let source = "`hello \\` world`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_escaped_dollar() {
        let source = "`price: \\$100`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_unicode_escape() {
        let source = "`\\u0041`"; // 'A'
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_hex_escape() {
        let source = "`\\x41`"; // 'A'
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_unterminated_template_string() {
        let source = "`hello world";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.template_string();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    #[test]
    fn test_unterminated_template_interpolation() {
        let source = "`hello ${name";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.template_string();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    #[test]
    fn test_template_string_with_comment_in_interpolation() {
        let source = "`result: ${a /* comment */ + b}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_with_line_comment_in_interpolation() {
        let source = "`result: ${a // comment\n+ b}`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }

    #[test]
    fn test_template_string_complex_nesting() {
        let source = "`a ${b({ c: `d ${e}` })} f`";
        let mut lexer = Lexer::new(source);
        
        assert!(lexer.template_string().is_ok());
        assert_eq!(lexer.position(), source.len());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Helper: Escape a string for use in JavaScript string literals
    fn escape_for_js_string(s: &str) -> String {
        let mut result = String::new();
        for ch in s.chars() {
            match ch {
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\u{0008}' => result.push_str("\\b"),
                '\u{000B}' => result.push_str("\\v"),
                '\u{000C}' => result.push_str("\\f"),
                '\0' => result.push_str("\\0"),
                '\\' => result.push_str("\\\\"),
                '\'' => result.push_str("\\'"),
                '"' => result.push_str("\\\""),
                c if c.is_control() || c as u32 > 0x10FFFF => {
                    // Use unicode escape for control characters
                    result.push_str(&format!("\\u{{{:x}}}", c as u32));
                }
                c => result.push(c),
            }
        }
        result
    }

    // Arbitrary string generator for testing
    fn arb_test_string() -> impl Strategy<Value = String> {
        prop::string::string_regex("[\\x20-\\x7E\\u{80}-\\u{FFFF}]{0,20}").unwrap()
    }

    // Arbitrary escape sequence generator
    fn arb_escape_sequence() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("\\n".to_string()),
            Just("\\r".to_string()),
            Just("\\t".to_string()),
            Just("\\b".to_string()),
            Just("\\v".to_string()),
            Just("\\f".to_string()),
            Just("\\0".to_string()),
            Just("\\\\".to_string()),
            Just("\\'".to_string()),
            Just("\\\"".to_string()),
            // Hex escapes
            (0x00u8..=0x7F).prop_map(|n| format!("\\x{:02x}", n)),
            // Unicode escapes
            (0x0000u32..=0xFFFF).prop_map(|n| format!("\\u{:04x}", n)),
            // Unicode brace escapes
            (0x0000u32..=0x10FFFF).prop_map(|n| format!("\\u{{{:x}}}", n)),
        ]
    }

    // Arbitrary string with escapes
    fn arb_string_with_escapes() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                arb_escape_sequence(),
                prop::string::string_regex("[^'\"\\\\\\n\\r]{1,5}").unwrap(),
            ],
            0..10
        ).prop_map(|parts| parts.join(""))
    }

    // Arbitrary quote character
    fn arb_quote() -> impl Strategy<Value = u8> {
        prop::sample::select(vec![b'\'', b'"'])
    }

    // Feature: es-module-lexer-rs, Property 6: 字符串转义 Round-Trip
    // Validates: Requirements 5.2, 5.3, 8.3
    proptest! {
        #[test]
        fn prop_string_escape_roundtrip(
            s in arb_test_string(),
            quote in arb_quote()
        ) {
            // Escape the string for JavaScript
            let escaped = escape_for_js_string(&s);
            let quote_char = quote as char;
            let source = format!("{}{}{}", quote_char, escaped, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.read_string(quote);
            
            // Should successfully parse
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            
            let parsed = result.unwrap();
            
            // The parsed string should match the original
            prop_assert_eq!(parsed, s, "Round-trip failed for: {}", source);
        }

        #[test]
        fn prop_simple_escape_sequences(
            escape_seq in arb_escape_sequence(),
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let source = format!("{}{}{}", quote_char, escape_seq, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.read_string(quote);
            
            // Should successfully parse any valid escape sequence
            prop_assert!(result.is_ok(), "Failed to parse escape: {}", source);
        }

        #[test]
        fn prop_string_with_multiple_escapes(
            s in arb_string_with_escapes(),
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let source = format!("{}{}{}", quote_char, s, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.read_string(quote);
            
            // Should successfully parse strings with multiple escapes
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
        }

        #[test]
        fn prop_hex_escape_values(
            value in 0x00u8..=0xFF,
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let source = format!("{}\\x{:02x}{}", quote_char, value, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.read_string(quote).unwrap();
            
            // Should decode to the correct character
            let expected = char::from_u32(value as u32).unwrap();
            prop_assert_eq!(result, expected.to_string());
        }

        #[test]
        fn prop_unicode_escape_values(
            value in 0x0000u32..=0xFFFF,
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let source = format!("{}\\u{:04x}{}", quote_char, value, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.read_string(quote).unwrap();
            
            // Should decode to the correct character
            let expected = char::from_u32(value).unwrap_or('\u{FFFD}');
            prop_assert_eq!(result, expected.to_string());
        }

        #[test]
        fn prop_unicode_brace_escape_values(
            value in 0x0000u32..=0x10FFFF,
            quote in arb_quote()
        ) {
            // Skip surrogate pairs which are invalid
            prop_assume!(value < 0xD800 || value > 0xDFFF);
            
            let quote_char = quote as char;
            let source = format!("{}\\u{{{:x}}}{}", quote_char, value, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.read_string(quote).unwrap();
            
            // Should decode to the correct character
            let expected = char::from_u32(value).unwrap_or('\u{FFFD}');
            prop_assert_eq!(result, expected.to_string());
        }

        #[test]
        fn prop_line_continuation_removed(
            before in prop::string::string_regex("[^\\\\\\n\\r'\"]{0,10}").unwrap(),
            after in prop::string::string_regex("[^\\\\\\n\\r'\"]{0,10}").unwrap(),
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let source = format!("{}{}\\{}{}{}", quote_char, before, '\n', after, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.read_string(quote).unwrap();
            
            // Line continuation should be removed
            let expected = format!("{}{}", before, after);
            prop_assert_eq!(result, expected);
        }

        #[test]
        fn prop_escaped_quotes_dont_terminate(
            content in prop::string::string_regex("[^'\"\\\\\\n\\r]{0,10}").unwrap(),
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let escaped_quote = format!("\\{}", quote_char);
            let source = format!("{}{}{}{}{}", quote_char, content, escaped_quote, content, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.read_string(quote).unwrap();
            
            // Should include the escaped quote in the result
            let expected = format!("{}{}{}", content, quote_char, content);
            prop_assert_eq!(result, expected);
        }
    }

    // Feature: es-module-lexer-rs, Property 9: 字符串解析完整性
    // Validates: Requirements 8.1, 8.2
    proptest! {
        #[test]
        fn prop_string_parsing_completeness_single_quote(
            content in prop::string::string_regex("[^'\\\\\\n\\r]{0,50}").unwrap()
        ) {
            let source = format!("'{}'", content);
            let mut lexer = Lexer::new(&source);
            
            // Should successfully scan to closing quote
            let result = lexer.string_literal(b'\'');
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            
            // Position should be after the closing quote
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_string_parsing_completeness_double_quote(
            content in prop::string::string_regex("[^\"\\\\\\n\\r]{0,50}").unwrap()
        ) {
            let source = format!("\"{}\"", content);
            let mut lexer = Lexer::new(&source);
            
            // Should successfully scan to closing quote
            let result = lexer.string_literal(b'"');
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            
            // Position should be after the closing quote
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_string_with_escaped_quotes_completes(
            parts in prop::collection::vec(
                prop_oneof![
                    prop::string::string_regex("[^'\"\\\\\\n\\r]{1,5}").unwrap(),
                    Just("\\'".to_string()),
                ],
                1..10
            ),
            quote in arb_quote()
        ) {
            let content = parts.join("");
            let quote_char = quote as char;
            let source = format!("{}{}{}", quote_char, content, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.string_literal(quote);
            
            // Should successfully scan to closing quote despite escaped quotes
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_unterminated_string_errors(
            content in prop::string::string_regex("[^'\"\\\\\\n\\r]{0,20}").unwrap(),
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let source = format!("{}{}", quote_char, content); // No closing quote
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.string_literal(quote);
            
            // Should return an error for unterminated string
            prop_assert!(result.is_err());
            prop_assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
        }

        #[test]
        fn prop_string_with_newline_errors(
            before in prop::string::string_regex("[^'\"\\\\\\n\\r]{0,10}").unwrap(),
            after in prop::string::string_regex("[^'\"\\\\\\n\\r]{0,10}").unwrap(),
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let source = format!("{}{}\n{}{}", quote_char, before, after, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.string_literal(quote);
            
            // Should return an error for unescaped newline
            prop_assert!(result.is_err());
            prop_assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
        }

        #[test]
        fn prop_empty_string_completes(
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let source = format!("{}{}", quote_char, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.string_literal(quote);
            
            // Empty strings should parse successfully
            prop_assert!(result.is_ok());
            prop_assert_eq!(lexer.position(), 2);
        }

        #[test]
        fn prop_string_with_all_escape_types(
            simple_escapes in prop::collection::vec(
                prop::sample::select(vec!["\\n", "\\r", "\\t", "\\b", "\\v", "\\f", "\\0", "\\\\", "\\'", "\\\""]),
                0..5
            ),
            quote in arb_quote()
        ) {
            let content = simple_escapes.join("");
            let quote_char = quote as char;
            let source = format!("{}{}{}", quote_char, content, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.string_literal(quote);
            
            // Should handle all simple escape types
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_string_literal_and_read_string_consistent(
            content in prop::string::string_regex("[^'\"\\\\\\n\\r]{0,20}").unwrap(),
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let source = format!("{}{}{}", quote_char, content, quote_char);
            
            // Test string_literal
            let mut lexer1 = Lexer::new(&source);
            let result1 = lexer1.string_literal(quote);
            let pos1 = lexer1.position();
            
            // Test read_string
            let mut lexer2 = Lexer::new(&source);
            let result2 = lexer2.read_string(quote);
            let pos2 = lexer2.position();
            
            // Both should succeed and end at the same position
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());
            prop_assert_eq!(pos1, pos2);
            
            // read_string should return the content
            prop_assert_eq!(result2.unwrap(), content);
        }

        #[test]
        fn prop_mixed_quotes_dont_interfere(
            content in prop::string::string_regex("[^'\"\\\\\\n\\r]{0,10}").unwrap(),
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let other_quote = if quote == b'\'' { '"' } else { '\'' };
            
            // String with the other quote type inside
            let source = format!("{}{}{}{}{}", quote_char, content, other_quote, content, quote_char);
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.string_literal(quote);
            
            // Should parse successfully - other quote type doesn't terminate
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }
    }

    // ===== Template String Property Tests =====

    // Arbitrary template string content (no backticks or unescaped ${)
    fn arb_template_content() -> impl Strategy<Value = String> {
        prop::string::string_regex("[^`$\\\\\\x00-\\x1F]{0,20}").unwrap()
    }

    // Arbitrary simple expression (identifier or number)
    fn arb_simple_expression() -> impl Strategy<Value = String> {
        prop_oneof![
            prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]{0,10}").unwrap(),
            (0..1000i32).prop_map(|n| n.to_string()),
        ]
    }

    // Arbitrary template string with interpolations
    #[allow(dead_code)]
    fn arb_template_with_interpolations() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                arb_template_content(),
                arb_simple_expression().prop_map(|expr| format!("${{{}}}", expr)),
            ],
            0..5
        ).prop_map(|parts| format!("`{}`", parts.join("")))
    }

    // Arbitrary nested template string (limited depth)
    #[allow(dead_code)]
    fn arb_nested_template(depth: u32) -> impl Strategy<Value = String> {
        if depth == 0 {
            arb_template_content().prop_map(|s| format!("`{}`", s)).boxed()
        } else {
            prop_oneof![
                arb_template_content().prop_map(|s| format!("`{}`", s)),
                (arb_template_content(), arb_simple_expression())
                    .prop_map(|(before, expr)| format!("`{}${{{}}}`", before, expr)),
                (arb_template_content(), arb_nested_template(depth - 1), arb_template_content())
                    .prop_map(|(before, nested, after)| format!("`{}${{{}}}{}` ", before, nested, after)),
            ].boxed()
        }
    }

    // Feature: es-module-lexer-rs, Property 10: 模板字符串嵌套处理
    // Validates: Requirements 8.4, 8.5
    proptest! {
        #[test]
        fn prop_template_string_simple_parsing(
            content in arb_template_content()
        ) {
            let source = format!("`{}`", content);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_string_with_single_interpolation(
            before in arb_template_content(),
            expr in arb_simple_expression(),
            after in arb_template_content()
        ) {
            let source = format!("`{}${{{}}}{}` ", before, expr, after);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            // Account for trailing space in format string
            prop_assert_eq!(lexer.position(), source.len() - 1);
        }

        #[test]
        fn prop_template_string_with_multiple_interpolations(
            parts in prop::collection::vec(
                (arb_template_content(), arb_simple_expression()),
                1..5
            ),
            final_content in arb_template_content()
        ) {
            let mut source = String::from("`");
            for (content, expr) in parts {
                source.push_str(&content);
                source.push_str(&format!("${{{}}}", expr));
            }
            source.push_str(&final_content);
            source.push('`');
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.template_string();
            
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_string_nested_one_level(
            outer_before in arb_template_content(),
            inner_content in arb_template_content(),
            outer_after in arb_template_content()
        ) {
            let source = format!("`{}${{`{}`}}{}` ", outer_before, inner_content, outer_after);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            // Account for trailing space in format string
            prop_assert_eq!(lexer.position(), source.len() - 1);
        }

        #[test]
        fn prop_template_string_nested_two_levels(
            l1_before in arb_template_content(),
            l2_before in arb_template_content(),
            l3_content in arb_template_content(),
            l2_after in arb_template_content(),
            l1_after in arb_template_content()
        ) {
            let source = format!(
                "`{}${{`{}${{`{}`}}{}` }}{}` ",
                l1_before, l2_before, l3_content, l2_after, l1_after
            );
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            // Account for trailing space in format string
            prop_assert_eq!(lexer.position(), source.len() - 1);
        }

        #[test]
        fn prop_template_string_with_object_literal(
            key in prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]{0,5}").unwrap(),
            value in arb_simple_expression()
        ) {
            let source = format!("`result: ${{{{ {}: {} }}}}`", key, value);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_string_with_array_literal(
            values in prop::collection::vec(arb_simple_expression(), 0..5)
        ) {
            let array = format!("[{}]", values.join(", "));
            let source = format!("`array: ${{{}}}`", array);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_string_with_function_call(
            func_name in prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]{0,10}").unwrap(),
            args in prop::collection::vec(arb_simple_expression(), 0..3)
        ) {
            let call = format!("{}({})", func_name, args.join(", "));
            let source = format!("`result: ${{{}}}`", call);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_string_with_string_in_interpolation(
            outer_content in arb_template_content(),
            inner_string in prop::string::string_regex("[^'\"\\\\\\n\\r]{0,10}").unwrap(),
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            let source = format!("`{}${{{}{}{}}}`", outer_content, quote_char, inner_string, quote_char);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_string_empty_interpolation(
            before in arb_template_content(),
            after in arb_template_content()
        ) {
            let source = format!("`{}${{}}{}` ", before, after);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            // Account for trailing space in format string
            prop_assert_eq!(lexer.position(), source.len() - 1);
        }

        #[test]
        fn prop_template_string_with_newlines(
            lines in prop::collection::vec(arb_template_content(), 1..5)
        ) {
            let source = format!("`{}`", lines.join("\n"));
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_string_unterminated_errors(
            content in arb_template_content()
        ) {
            let source = format!("`{}", content); // No closing backtick
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_err());
            prop_assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
        }

        #[test]
        fn prop_template_string_unterminated_interpolation_errors(
            before in arb_template_content(),
            expr in arb_simple_expression()
        ) {
            let source = format!("`{}${{{}", before, expr); // No closing } or `
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_err());
            prop_assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
        }

        #[test]
        fn prop_template_string_tracks_nesting_level(
            depth in 1..4usize
        ) {
            // Build nested template strings
            let mut source = String::new();
            for _ in 0..depth {
                source.push_str("`a${{");
            }
            for _ in 0..depth {
                source.push_str("}}`");
            }
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.template_string();
            
            prop_assert!(result.is_ok(), "Failed to parse depth {}: {}", depth, source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_string_with_escaped_sequences(
            escapes in prop::collection::vec(
                prop::sample::select(vec!["\\n", "\\r", "\\t", "\\\\", "\\`", "\\$"]),
                0..5
            )
        ) {
            let content = escapes.join("");
            let source = format!("`{}`", content);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }
    }

    // Feature: es-module-lexer-rs, Property 13: 模板字符串括号匹配
    // Validates: Requirements 7.6
    
    #[test]
    fn test_template_brace_matching_empty_expression() {
        let source = "`${}`";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.template_string();
        assert!(result.is_ok(), "Failed to parse: {}", source);
        assert_eq!(lexer.position(), source.len());
    }
    
    proptest! {
        #[test]
        fn prop_template_brace_matching_simple(
            expr in arb_simple_expression()
        ) {
            let source = format!("`${{{}}}`", expr);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_nested_braces(
            depth in 1..5usize
        ) {
            // Build expression with nested braces: ${{ ... }}
            let mut source = String::from("`${");
            for _ in 0..depth {
                source.push('{');
            }
            source.push_str("a");
            for _ in 0..depth {
                source.push('}');
            }
            source.push_str("}`");
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.template_string();
            
            prop_assert!(result.is_ok(), "Failed to parse depth {}: {}", depth, source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_multiple_expressions(
            exprs in prop::collection::vec(arb_simple_expression(), 1..5)
        ) {
            let mut source = String::from("`");
            for expr in exprs {
                source.push_str(&format!("${{{}}}", expr));
            }
            source.push('`');
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.template_string();
            
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_with_nested_template(
            outer_expr in arb_simple_expression(),
            inner_expr in arb_simple_expression()
        ) {
            let source = format!("`${{{}+`${{{}}}`}}`", outer_expr, inner_expr);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_with_object_literal(
            keys in prop::collection::vec(
                prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]{0,5}").unwrap(),
                1..3
            ),
            values in prop::collection::vec(arb_simple_expression(), 1..3)
        ) {
            prop_assume!(keys.len() == values.len());
            
            let obj_content = keys.iter()
                .zip(values.iter())
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            
            let source = format!("`${{{{ {} }}}}`", obj_content);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_with_array_literal(
            elements in prop::collection::vec(arb_simple_expression(), 0..5)
        ) {
            let array_content = elements.join(", ");
            let source = format!("`${{[{}]}}`", array_content);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_with_function_call(
            func_name in prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]{0,10}").unwrap(),
            args in prop::collection::vec(arb_simple_expression(), 0..3)
        ) {
            let call = format!("{}({})", func_name, args.join(", "));
            let source = format!("`${{{}}}`", call);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_complex_expression(
            a in arb_simple_expression(),
            b in arb_simple_expression(),
            c in arb_simple_expression()
        ) {
            // Complex expression with multiple operators and parentheses
            let source = format!("`${{({} + {}) * {}}}`", a, b, c);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_with_ternary(
            cond in arb_simple_expression(),
            true_val in arb_simple_expression(),
            false_val in arb_simple_expression()
        ) {
            let source = format!("`${{{}?{}:{}}}`", cond, true_val, false_val);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_deeply_nested(
            depth in 1..4usize
        ) {
            // Build deeply nested template with braces
            let mut source = String::from("`");
            for i in 0..depth {
                source.push_str("${");
                if i < depth - 1 {
                    source.push_str("`");
                }
            }
            source.push_str("x");
            for i in 0..depth {
                if i > 0 {
                    source.push('`');
                }
                source.push('}');
            }
            source.push('`');
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.template_string();
            
            prop_assert!(result.is_ok(), "Failed to parse depth {}: {}", depth, source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_with_string_containing_braces(
            content in prop::string::string_regex("[^'\"\\\\\\n\\r]{0,10}").unwrap(),
            quote in arb_quote()
        ) {
            let quote_char = quote as char;
            // String inside template expression that contains brace-like characters
            let source = format!("`${{{}{{{}}}{}}}`", quote_char, content, quote_char);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_with_comments(
            expr in arb_simple_expression()
        ) {
            let source = format!("`${{/* comment */ {}}}`", expr);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.template_string();
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_sequential_expressions(
            n in 1..5usize
        ) {
            let mut source = String::from("`");
            for i in 0..n {
                source.push_str(&format!("${{a{}}}", i));
            }
            source.push('`');
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.template_string();
            
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }

        #[test]
        fn prop_template_brace_matching_interleaved_content(
            parts in prop::collection::vec(
                (arb_template_content(), arb_simple_expression()),
                1..5
            )
        ) {
            let mut source = String::from("`");
            for (content, expr) in parts {
                source.push_str(&content);
                source.push_str(&format!("${{{}}}", expr));
            }
            source.push('`');
            
            let mut lexer = Lexer::new(&source);
            let result = lexer.template_string();
            
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);
            prop_assert_eq!(lexer.position(), source.len());
        }
    }
}
