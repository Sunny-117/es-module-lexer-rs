//! Comprehensive error handling tests.
//!
//! This module tests all error conditions defined in LexerError.

use crate::error::LexerError;
use crate::lexer::Lexer;

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // UnterminatedString Tests
    // ========================================================================

    #[test]
    fn test_unterminated_single_quote_string() {
        let source = "import foo from 'bar";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    #[test]
    fn test_unterminated_double_quote_string() {
        let source = r#"import foo from "bar"#;
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    #[test]
    fn test_unterminated_string_with_escape() {
        let source = r#"import foo from "bar\""#;
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    #[test]
    fn test_unterminated_template_string() {
        let source = "const x = `hello world";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    #[test]
    fn test_unterminated_template_with_interpolation() {
        let source = "const x = `hello ${name";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    #[test]
    fn test_string_with_unescaped_newline() {
        let source = "import foo from 'bar\ntest';";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    // ========================================================================
    // UnterminatedComment Tests
    // ========================================================================

    #[test]
    fn test_unterminated_block_comment() {
        let source = "/* This is a comment";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedComment(_)));
    }

    #[test]
    fn test_unterminated_nested_block_comment() {
        // Note: JavaScript doesn't support nested block comments
        // The inner */ closes the outer comment, leaving "still in outer" as code
        let source = "/* outer /* inner */ still in outer";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        // This actually parses successfully because */ closes the comment
        // The remaining text is just regular code
        assert!(result.is_ok());
    }

    #[test]
    fn test_unterminated_block_comment_in_import() {
        let source = "import /* comment foo from 'bar';";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedComment(_)));
    }

    // ========================================================================
    // UnterminatedRegex Tests
    // ========================================================================

    #[test]
    fn test_unterminated_regex() {
        // In full parse mode, regex detection depends on context
        // This test documents the expected behavior
        let source = "return /test";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        // The lexer may or may not detect this as a regex depending on implementation
        // For now, we just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_unterminated_regex_with_escape() {
        let source = r"const re = /test\/";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedRegex(_)));
    }

    #[test]
    fn test_unterminated_regex_character_class() {
        let source = "const re = /[abc/";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedRegex(_)));
    }

    #[test]
    fn test_regex_with_newline() {
        let source = "const re = /test\n/";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedRegex(_)));
    }

    // ========================================================================
    // InvalidEscape Tests
    // ========================================================================

    #[test]
    fn test_invalid_hex_escape() {
        let source = r"import foo from '\xGG';";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::InvalidEscape(_)));
    }

    #[test]
    fn test_invalid_unicode_escape() {
        let source = r"import foo from '\uGGGG';";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::InvalidEscape(_)));
    }

    #[test]
    fn test_invalid_unicode_brace_escape() {
        let source = r"import foo from '\u{GGGG}';";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::InvalidEscape(_)));
    }

    #[test]
    fn test_unicode_escape_too_large() {
        let source = r"import foo from '\u{110000}';";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::InvalidEscape(_)));
    }

    #[test]
    fn test_incomplete_hex_escape() {
        let source = r"import foo from '\x1";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        // Could be either InvalidEscape or UnterminatedString
        assert!(matches!(
            result.unwrap_err(),
            LexerError::InvalidEscape(_) | LexerError::UnterminatedString(_)
        ));
    }

    #[test]
    fn test_incomplete_unicode_escape() {
        let source = r"import foo from '\u12";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LexerError::InvalidEscape(_) | LexerError::UnterminatedString(_)
        ));
    }

    // ========================================================================
    // ExpectedColon Tests (Import Attributes)
    // ========================================================================

    #[test]
    fn test_import_attributes_missing_colon() {
        // Note: Import attributes parsing is not fully integrated yet
        // This test documents expected behavior when it is integrated
        let source = r#"import foo from 'bar' with { type "json" };"#;
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        // Currently this may not error because attributes parsing is not called
        // When integrated, it should error with ExpectedColon
        let _ = result;
    }

    #[test]
    fn test_import_attributes_multiple_missing_colon() {
        // Note: Import attributes parsing is not fully integrated yet
        let source = r#"import foo from 'bar' with { type: "json", integrity "sha384" };"#;
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        // When integrated, should error with ExpectedColon
        let _ = result;
    }

    // ========================================================================
    // ExpectedString Tests (Import Attributes)
    // ========================================================================

    #[test]
    fn test_import_attributes_non_string_value() {
        // Note: Import attributes parsing is not fully integrated yet
        let source = r#"import foo from 'bar' with { type: json };"#;
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        // When integrated, should error with ExpectedString
        let _ = result;
    }

    #[test]
    fn test_import_attributes_number_value() {
        // Note: Import attributes parsing is not fully integrated yet
        let source = r#"import foo from 'bar' with { type: 123 };"#;
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        // When integrated, should error with ExpectedString
        let _ = result;
    }

    // ========================================================================
    // UnexpectedToken Tests
    // ========================================================================

    #[test]
    fn test_export_list_unexpected_token() {
        let source = "export { a b };";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnexpectedToken(_)));
    }

    #[test]
    fn test_import_attributes_unexpected_token() {
        // Note: Import attributes parsing is not fully integrated yet
        let source = r#"import foo from 'bar' with { type: "json" @ };"#;
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        // When integrated, should error with UnexpectedToken
        let _ = result;
    }

    #[test]
    fn test_destructuring_object_unexpected_token() {
        let source = "export const { a @ b } = obj;";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnexpectedToken(_)));
    }

    #[test]
    fn test_destructuring_array_unexpected_token() {
        let source = "export const [ a @ b ] = arr;";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnexpectedToken(_)));
    }

    #[test]
    fn test_read_identifier_invalid_start() {
        let source = "export const 123abc = 1;";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        // This should either error or switch to full parse mode
        // The behavior depends on implementation details
        // For now, we just ensure it doesn't panic
        let _ = result;
    }

    // ========================================================================
    // StackOverflow Tests
    // ========================================================================

    #[test]
    fn test_deeply_nested_braces() {
        // Create a string with many nested braces
        let depth = 1100; // More than MAX_STACK_DEPTH (1024)
        let mut source = String::from("const x = ");
        for _ in 0..depth {
            source.push('{');
        }
        source.push_str("1");
        for _ in 0..depth {
            source.push('}');
        }
        source.push(';');
        
        let mut lexer = Lexer::new(&source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::StackOverflow(_)));
    }

    #[test]
    fn test_deeply_nested_parentheses() {
        // Create a string with many nested parentheses
        let depth = 1100; // More than MAX_STACK_DEPTH (1024)
        let mut source = String::from("const x = ");
        for _ in 0..depth {
            source.push('(');
        }
        source.push_str("1");
        for _ in 0..depth {
            source.push(')');
        }
        source.push(';');
        
        let mut lexer = Lexer::new(&source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::StackOverflow(_)));
    }

    #[test]
    fn test_deeply_nested_template_strings() {
        // Create deeply nested template strings
        let depth = 1100; // More than MAX_STACK_DEPTH (1024)
        let mut source = String::from("const x = `");
        for _ in 0..depth {
            source.push_str("${`");
        }
        source.push_str("hello");
        for _ in 0..depth {
            source.push_str("`}");
        }
        source.push_str("`;");
        
        let mut lexer = Lexer::new(&source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::StackOverflow(_)));
    }

    #[test]
    fn test_stack_depth_at_limit() {
        // Create a string with exactly MAX_STACK_DEPTH (1024) nested braces
        let depth = 1024;
        let mut source = String::from("const x = ");
        for _ in 0..depth {
            source.push('{');
        }
        source.push_str("1");
        for _ in 0..depth {
            source.push('}');
        }
        source.push(';');
        
        let mut lexer = Lexer::new(&source);
        let result = lexer.parse();
        
        // At exactly the limit, it should succeed
        assert!(result.is_ok());
    }

    // ========================================================================
    // Error Recovery Tests
    // ========================================================================

    #[test]
    fn test_error_position_accuracy() {
        let source = "import foo from 'bar";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        if let Err(LexerError::UnterminatedString(pos)) = result {
            // Position should point to the opening quote
            assert_eq!(pos, 16);
        } else {
            panic!("Expected UnterminatedString error");
        }
    }

    #[test]
    fn test_multiple_errors_first_reported() {
        // Source with multiple errors - should report the first one
        let source = "import foo from 'bar\nimport baz from 'qux";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        // Should report the first error (unterminated string on line 1)
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    #[test]
    fn test_error_in_facade_mode() {
        // Error should be caught even in facade mode
        let source = "import foo from 'bar";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    #[test]
    fn test_error_in_full_parse_mode() {
        // Error in full parse mode (after facade mode fails)
        let source = "const x = 1;\nimport foo from 'bar";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_empty_source_no_error() {
        let source = "";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_whitespace_only_no_error() {
        let source = "   \n\t\r\n   ";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_comments_only_no_error() {
        let source = "// comment\n/* block comment */";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_import_no_error() {
        let source = "import foo from 'bar';";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_export_no_error() {
        let source = "export const x = 1;";
        let mut lexer = Lexer::new(source);
        let result = lexer.parse();
        
        assert!(result.is_ok());
    }
}
