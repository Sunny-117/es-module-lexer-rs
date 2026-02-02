//! Comment and whitespace handling.

use crate::lexer::Lexer;
use crate::error::LexerError;

impl<'a> Lexer<'a> {
    /// Skip single-line comment (// ...).
    /// Assumes the current position is at the first '/'.
    #[inline]
    pub(crate) fn skip_line_comment(&mut self) -> Result<(), LexerError> {
        // Skip the '//' characters
        self.advance_by(2);
        
        // Skip until end of line or end of file
        while !self.is_at_end() {
            if let Some(ch) = self.peek() {
                if ch == b'\n' || ch == b'\r' {
                    break;
                }
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Skip multi-line comment (/* ... */).
    /// Assumes the current position is at the first '/'.
    #[inline]
    pub(crate) fn skip_block_comment(&mut self) -> Result<(), LexerError> {
        let start_pos = self.position();
        
        // Skip the '/*' characters
        self.advance_by(2);
        
        // Find the closing '*/'
        while !self.is_at_end() {
            if let Some(ch) = self.peek() {
                if ch == b'*' {
                    if let Some(next_ch) = self.peek_at(1) {
                        if next_ch == b'/' {
                            // Found closing */
                            self.advance_by(2);
                            return Ok(());
                        }
                    }
                }
                self.advance();
            } else {
                break;
            }
        }
        
        // Reached end of file without finding closing */
        Err(LexerError::UnterminatedComment(start_pos))
    }
    
    /// Skip comments and whitespace, returning the next non-whitespace character.
    /// 
    /// This method advances the position past any whitespace and comments,
    /// then returns the next character without consuming it.
    /// 
    /// # Arguments
    /// * `allow_regex` - Whether to allow regex literals (affects how '/' is handled)
    /// 
    /// # Returns
    /// The next non-whitespace, non-comment character, or 0 if at end of file.
    pub(crate) fn comment_whitespace(&mut self, _allow_regex: bool) -> Result<u8, LexerError> {
        loop {
            if self.is_at_end() {
                return Ok(0);
            }
            
            if let Some(ch) = self.peek() {
                match ch {
                    // Whitespace characters
                    b' ' | b'\t' | b'\n' | b'\r' | b'\x0B' | b'\x0C' => {
                        self.advance();
                    }
                    
                    // Potential comment start
                    b'/' => {
                        if let Some(next_ch) = self.peek_at(1) {
                            match next_ch {
                                b'/' => {
                                    // Single-line comment
                                    self.skip_line_comment()?;
                                }
                                b'*' => {
                                    // Multi-line comment
                                    self.skip_block_comment()?;
                                }
                                _ => {
                                    // Not a comment, return the '/'
                                    return Ok(ch);
                                }
                            }
                        } else {
                            // '/' at end of file
                            return Ok(ch);
                        }
                    }
                    
                    // Any other character - not whitespace or comment
                    _ => {
                        return Ok(ch);
                    }
                }
            } else {
                return Ok(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_skip_line_comment() {
        let source = "// this is a comment\ncode";
        let mut lexer = Lexer::new(source);
        
        lexer.skip_line_comment().unwrap();
        // Position should be at '\n' (position 20, not 21)
        assert_eq!(lexer.position(), 20);
        assert_eq!(lexer.peek(), Some(b'\n'));
    }
    
    #[test]
    fn test_skip_line_comment_at_eof() {
        let source = "// comment at end";
        let mut lexer = Lexer::new(source);
        
        lexer.skip_line_comment().unwrap();
        assert_eq!(lexer.position(), source.len());
        assert!(lexer.is_at_end());
    }
    
    #[test]
    fn test_skip_line_comment_with_special_chars() {
        let source = "// import 'foo'; export { bar }\ncode";
        let mut lexer = Lexer::new(source);
        
        lexer.skip_line_comment().unwrap();
        assert_eq!(lexer.peek(), Some(b'\n'));
    }
    
    #[test]
    fn test_skip_block_comment() {
        let source = "/* this is a comment */code";
        let mut lexer = Lexer::new(source);
        
        lexer.skip_block_comment().unwrap();
        assert_eq!(lexer.position(), 23); // Should be after '*/'
        assert_eq!(lexer.peek(), Some(b'c'));
    }
    
    #[test]
    fn test_skip_block_comment_multiline() {
        let source = "/* line 1\nline 2\nline 3 */code";
        let mut lexer = Lexer::new(source);
        
        lexer.skip_block_comment().unwrap();
        assert_eq!(lexer.peek(), Some(b'c'));
    }
    
    #[test]
    fn test_skip_block_comment_with_special_chars() {
        let source = "/* import 'foo'; export { bar } */code";
        let mut lexer = Lexer::new(source);
        
        lexer.skip_block_comment().unwrap();
        assert_eq!(lexer.peek(), Some(b'c'));
    }
    
    #[test]
    fn test_skip_block_comment_with_asterisks() {
        let source = "/* ** *** **** */code";
        let mut lexer = Lexer::new(source);
        
        lexer.skip_block_comment().unwrap();
        assert_eq!(lexer.peek(), Some(b'c'));
    }
    
    #[test]
    fn test_skip_block_comment_unterminated() {
        let source = "/* unterminated comment";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.skip_block_comment();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedComment(0)));
    }
    
    #[test]
    fn test_comment_whitespace_spaces() {
        let source = "   code";
        let mut lexer = Lexer::new(source);
        
        let ch = lexer.comment_whitespace(false).unwrap();
        assert_eq!(ch, b'c');
        assert_eq!(lexer.position(), 3);
    }
    
    #[test]
    fn test_comment_whitespace_tabs_and_newlines() {
        let source = "\t\n\r  code";
        let mut lexer = Lexer::new(source);
        
        let ch = lexer.comment_whitespace(false).unwrap();
        assert_eq!(ch, b'c');
    }
    
    #[test]
    fn test_comment_whitespace_line_comment() {
        let source = "// comment\ncode";
        let mut lexer = Lexer::new(source);
        
        let ch = lexer.comment_whitespace(false).unwrap();
        // After skipping comment and whitespace, should be at 'c'
        assert_eq!(ch, b'c');
    }
    
    #[test]
    fn test_comment_whitespace_block_comment() {
        let source = "/* comment */code";
        let mut lexer = Lexer::new(source);
        
        let ch = lexer.comment_whitespace(false).unwrap();
        assert_eq!(ch, b'c');
    }
    
    #[test]
    fn test_comment_whitespace_mixed() {
        let source = "  /* comment */  // line comment\n  code";
        let mut lexer = Lexer::new(source);
        
        let ch = lexer.comment_whitespace(false).unwrap();
        // After skipping all comments and whitespace, should be at 'c'
        assert_eq!(ch, b'c');
    }
    
    #[test]
    fn test_comment_whitespace_at_eof() {
        let source = "   ";
        let mut lexer = Lexer::new(source);
        
        let ch = lexer.comment_whitespace(false).unwrap();
        assert_eq!(ch, 0);
        assert!(lexer.is_at_end());
    }
    
    #[test]
    fn test_comment_whitespace_slash_not_comment() {
        let source = "/code";
        let mut lexer = Lexer::new(source);
        
        let ch = lexer.comment_whitespace(false).unwrap();
        assert_eq!(ch, b'/');
        assert_eq!(lexer.position(), 0); // Should not advance
    }
    
    #[test]
    fn test_comment_whitespace_multiple_comments() {
        let source = "// comment 1\n/* comment 2 */  // comment 3\ncode";
        let mut lexer = Lexer::new(source);
        
        let ch = lexer.comment_whitespace(false).unwrap();
        // After skipping all comments and whitespace, should be at 'c'
        assert_eq!(ch, b'c');
    }
    
    #[test]
    fn test_comment_with_import_export_keywords() {
        let source = "// import foo from 'bar'\n/* export { baz } */\nactual_code";
        let mut lexer = Lexer::new(source);
        
        let ch = lexer.comment_whitespace(false).unwrap();
        // After skipping all comments and whitespace, should be at 'a'
        assert_eq!(ch, b'a');
    }
}


#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    
    // Generator for valid single-line comments
    fn arb_line_comment() -> impl Strategy<Value = String> {
        // Generate ASCII-only string that doesn't contain newlines
        prop::string::string_regex("[a-zA-Z0-9 _.,;:!?-]*").unwrap()
            .prop_map(|content| format!("// {}\n", content))
    }
    
    // Generator for valid multi-line comments
    fn arb_block_comment() -> impl Strategy<Value = String> {
        // Generate ASCII-only content that doesn't contain */
        prop::string::string_regex("[a-zA-Z0-9 \n\r_.,;:!?-]*").unwrap()
            .prop_filter("doesn't contain */", |s| !s.contains("*/"))
            .prop_map(|content| format!("/* {} */", content))
    }
    
    // Generator for whitespace
    fn arb_whitespace() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop::sample::select(vec![' ', '\t', '\n', '\r']),
            0..10
        ).prop_map(|chars| chars.into_iter().collect())
    }
    
    // Generator for code that looks like import/export but is in comments
    fn arb_comment_with_keywords() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("// import foo from 'bar'\n".to_string()),
            Just("// export { baz }\n".to_string()),
            Just("/* import * as x from 'y' */".to_string()),
            Just("/* export default function() {} */".to_string()),
            arb_line_comment(),
            arb_block_comment(),
        ]
    }
    
    // Feature: es-module-lexer-rs, Property 11: 注释跳过完整性
    // Validates: Requirements 9.1, 9.2, 9.3, 9.4
    proptest! {
        #[test]
        fn prop_comment_skipping_completeness(
            comments in prop::collection::vec(arb_comment_with_keywords(), 1..5),
            code in prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]*").unwrap()
        ) {
            // Build source with comments followed by actual code
            let mut source = String::new();
            for comment in &comments {
                source.push_str(comment);
                source.push('\n');
            }
            source.push_str(&code);
            
            let mut lexer = Lexer::new(&source);
            
            // comment_whitespace should skip all comments and whitespace
            let ch = lexer.comment_whitespace(false).unwrap();
            
            // Should return the first character of the actual code
            if !code.is_empty() {
                prop_assert_eq!(ch, code.as_bytes()[0]);
            } else {
                prop_assert_eq!(ch, 0);
            }
        }
        
        #[test]
        fn prop_line_comment_never_crosses_newline(
            comment_content in prop::string::string_regex("[^\n\r]*").unwrap(),
            code in prop::string::string_regex("[a-zA-Z]+").unwrap()
        ) {
            let source = format!("// {}\n{}", comment_content, code);
            let mut lexer = Lexer::new(&source);
            
            lexer.skip_line_comment().unwrap();
            
            // Should stop at newline
            prop_assert_eq!(lexer.peek(), Some(b'\n'));
        }
        
        #[test]
        fn prop_block_comment_handles_asterisks(
            asterisks in prop::collection::vec(Just('*'), 0..10),
            content in prop::string::string_regex("[^*]+").unwrap()
        ) {
            let asterisk_str: String = asterisks.into_iter().collect();
            let source = format!("/* {} {} */code", asterisk_str, content);
            let mut lexer = Lexer::new(&source);
            
            lexer.skip_block_comment().unwrap();
            
            // Should be at 'c' after the comment
            prop_assert_eq!(lexer.peek(), Some(b'c'));
        }
        
        #[test]
        fn prop_comment_whitespace_skips_mixed(
            whitespace1 in arb_whitespace(),
            comment1 in arb_comment_with_keywords(),
            whitespace2 in arb_whitespace(),
            comment2 in arb_comment_with_keywords(),
            whitespace3 in arb_whitespace(),
            code in prop::string::string_regex("[a-zA-Z]+").unwrap()
        ) {
            let source = format!(
                "{}{}{}{}{}{}",
                whitespace1, comment1, whitespace2, comment2, whitespace3, code
            );
            
            let mut lexer = Lexer::new(&source);
            let ch = lexer.comment_whitespace(false).unwrap();
            
            // Should skip all whitespace and comments, returning first char of code
            if !code.is_empty() {
                prop_assert_eq!(ch, code.as_bytes()[0]);
            }
        }
        
        #[test]
        fn prop_unterminated_block_comment_errors(
            content in prop::string::string_regex("[^*]+").unwrap()
        ) {
            let source = format!("/* {}", content);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.skip_block_comment();
            
            // Should return an error
            prop_assert!(result.is_err());
            prop_assert!(matches!(result.unwrap_err(), LexerError::UnterminatedComment(_)));
        }
        
        #[test]
        fn prop_comment_with_import_export_ignored(
            keyword in prop::sample::select(vec!["import", "export"]),
            rest in prop::string::string_regex("[a-zA-Z0-9 _]*").unwrap()
        ) {
            let source = format!("// {} {}\nactual_code", keyword, rest);
            let mut lexer = Lexer::new(&source);
            
            let ch = lexer.comment_whitespace(false).unwrap();
            
            // Should skip the comment and return 'a' from "actual_code"
            prop_assert_eq!(ch, b'a');
        }
    }
}
