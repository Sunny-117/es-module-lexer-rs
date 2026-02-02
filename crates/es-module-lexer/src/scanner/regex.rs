//! Regular expression scanning.

use crate::lexer::Lexer;
use crate::error::LexerError;

impl<'a> Lexer<'a> {
    /// Scan a regular expression literal.
    /// 
    /// Handles:
    /// - Regular expression body with escape sequences
    /// - Character classes [...]
    /// - Flags (g, i, m, s, u, y, d)
    pub(crate) fn regular_expression(&mut self) -> Result<(), LexerError> {
        let start_pos = self.position();
        
        // Skip the opening '/'
        self.advance();
        
        while !self.is_at_end() {
            let ch = self.peek().unwrap();
            
            if ch == b'/' {
                // End of regex, now scan flags
                self.advance();
                self.scan_regex_flags();
                return Ok(());
            }
            
            if ch == b'[' {
                // Character class
                self.advance();
                self.regex_character_class()?;
            } else if ch == b'\\' {
                // Escape sequence - skip next character
                self.advance();
                if !self.is_at_end() {
                    self.advance();
                }
            } else if ch == b'\n' || ch == b'\r' {
                // Newline in regex is a syntax error
                return Err(LexerError::UnterminatedRegex(start_pos));
            } else {
                // Regular character
                self.advance();
            }
        }
        
        // Reached end without finding closing '/'
        Err(LexerError::UnterminatedRegex(start_pos))
    }
    
    /// Scan a character class [...] inside a regular expression.
    fn regex_character_class(&mut self) -> Result<(), LexerError> {
        let start_pos = self.position() - 1; // We've already advanced past '['
        
        while !self.is_at_end() {
            let ch = self.peek().unwrap();
            
            if ch == b']' {
                // End of character class
                self.advance();
                return Ok(());
            }
            
            if ch == b'\\' {
                // Escape sequence - skip next character
                self.advance();
                if !self.is_at_end() {
                    self.advance();
                }
            } else if ch == b'\n' || ch == b'\r' {
                // Newline in character class is a syntax error
                return Err(LexerError::UnterminatedRegex(start_pos));
            } else {
                // Regular character
                self.advance();
            }
        }
        
        // Reached end without finding closing ']'
        Err(LexerError::UnterminatedRegex(start_pos))
    }
    
    /// Scan regex flags after the closing '/'.
    /// Flags: g (global), i (ignoreCase), m (multiline), s (dotAll), 
    ///        u (unicode), y (sticky), d (hasIndices)
    fn scan_regex_flags(&mut self) {
        while !self.is_at_end() {
            let ch = self.peek().unwrap();
            
            // Check if it's a valid flag character
            if matches!(ch, b'g' | b'i' | b'm' | b's' | b'u' | b'y' | b'd') {
                self.advance();
            } else if ch.is_ascii_alphabetic() {
                // Invalid flag, but continue scanning to consume it
                self.advance();
            } else {
                // Not a flag character, stop
                break;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_regex() {
        let source = "/test/";
        let mut lexer = Lexer::new(source);
        assert!(lexer.regular_expression().is_ok());
        assert_eq!(lexer.position(), 6);
    }
    
    #[test]
    fn test_regex_with_flags() {
        let source = "/test/gi";
        let mut lexer = Lexer::new(source);
        assert!(lexer.regular_expression().is_ok());
        assert_eq!(lexer.position(), 8);
    }
    
    #[test]
    fn test_regex_with_all_flags() {
        let source = "/test/gimsuy";
        let mut lexer = Lexer::new(source);
        assert!(lexer.regular_expression().is_ok());
        assert_eq!(lexer.position(), 12);
    }
    
    #[test]
    fn test_regex_with_escape() {
        let source = r"/te\/st/";
        let mut lexer = Lexer::new(source);
        assert!(lexer.regular_expression().is_ok());
        assert_eq!(lexer.position(), 8);
    }
    
    #[test]
    fn test_regex_with_character_class() {
        let source = "/[abc]/";
        let mut lexer = Lexer::new(source);
        assert!(lexer.regular_expression().is_ok());
        assert_eq!(lexer.position(), 7);
    }
    
    #[test]
    fn test_regex_with_escaped_bracket_in_class() {
        let source = r"/[\]]/";
        let mut lexer = Lexer::new(source);
        assert!(lexer.regular_expression().is_ok());
        assert_eq!(lexer.position(), 6);
    }
    
    #[test]
    fn test_regex_with_nested_brackets() {
        let source = "/[a-z[0-9]]/";
        let mut lexer = Lexer::new(source);
        assert!(lexer.regular_expression().is_ok());
        assert_eq!(lexer.position(), 12);
    }
    
    #[test]
    fn test_unterminated_regex() {
        let source = "/test";
        let mut lexer = Lexer::new(source);
        let result = lexer.regular_expression();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedRegex(_)));
    }
    
    #[test]
    fn test_regex_with_newline() {
        let source = "/test\n/";
        let mut lexer = Lexer::new(source);
        let result = lexer.regular_expression();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedRegex(_)));
    }
    
    #[test]
    fn test_unterminated_character_class() {
        let source = "/[abc/";
        let mut lexer = Lexer::new(source);
        let result = lexer.regular_expression();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedRegex(_)));
    }
    
    #[test]
    fn test_complex_regex() {
        let source = r"/^[a-zA-Z_$][a-zA-Z0-9_$]*$/i";
        let mut lexer = Lexer::new(source);
        assert!(lexer.regular_expression().is_ok());
        assert_eq!(lexer.position(), 29);
    }
}
