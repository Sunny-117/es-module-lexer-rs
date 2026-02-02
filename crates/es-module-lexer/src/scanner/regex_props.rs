//! Property-based tests for regex vs division operator context judgment.
//!
//! Feature: es-module-lexer-rs
//! Property 8: 正则表达式 vs 除法运算符上下文判断
//! Validates Requirements: 6.1, 6.2, 6.3, 6.4, 6.5

use crate::lexer::Lexer;
use proptest::prelude::*;

/// Generate expression punctuators that should precede a regex
fn arb_expression_punctuator() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("!"),
        Just("%"),
        Just("&"),
        Just("("),
        Just("*"),
        Just("+"),
        Just(","),
        Just("-"),
        Just(":"),
        Just(";"),
        Just("<"),
        Just("="),
        Just(">"),
        Just("?"),
        Just("["),
        Just("^"),
        Just("{"),
        Just("|"),
        Just("~"),
    ]
}

/// Generate expression keywords that should precede a regex
fn arb_expression_keyword() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("return"),
        Just("throw"),
        Just("typeof"),
        Just("void"),
        Just("yield"),
        Just("await"),
        Just("delete"),
        Just("new"),
        Just("instanceof"),
        Just("case"),
        Just("else"),
        Just("do"),
        Just("in"),
        Just("break"),
        Just("continue"),
        Just("debugger"),
    ]
}

/// Generate contexts where '/' should be treated as a regex
fn arb_regex_context() -> impl Strategy<Value = String> {
    prop_oneof![
        // Expression punctuator followed by /
        arb_expression_punctuator().prop_map(|p| format!("{} /test/", p)),
        // Expression keyword followed by /
        arb_expression_keyword().prop_map(|k| format!("{} /test/", k)),
        // Start of file
        Just("/test/".to_string()),
        // After newline
        Just("\n/test/".to_string()),
        // Paren keywords: while, for, if
        Just("while (x) /test/".to_string()),
        Just("for (i) /test/".to_string()),
        Just("if (x) /test/".to_string()),
    ]
}

/// Generate contexts where '/' should be treated as division
fn arb_division_context() -> impl Strategy<Value = String> {
    prop_oneof![
        // After identifier
        Just("x /2".to_string()),
        Just("foo /bar".to_string()),
        // After number
        Just("10 /2".to_string()),
        Just("3.14 /2".to_string()),
        // After closing paren (not from while/for/if)
        Just("(x + y) /2".to_string()),
        // After property access
        Just("obj.prop /2".to_string()),
        // After array access
        Just("arr[0] /2".to_string()),
    ]
}

proptest! {
    /// Property 8: Regex vs Division Context Judgment
    /// 
    /// For any code containing '/', when '/' is preceded by an expression punctuator,
    /// expression keyword, or specific context ')' or '}', it should be parsed as a
    /// regular expression; when preceded by an identifier or number, it should be
    /// parsed as a division operator.
    ///
    /// Validates Requirements: 6.1, 6.2, 6.3, 6.4, 6.5
    #[test]
    fn prop_regex_context_detection(source in arb_regex_context()) {
        let mut lexer = Lexer::new(&source);
        
        // The lexer should successfully parse this as containing a regex
        // We can't directly test the internal state, but we can verify
        // that the lexer doesn't treat it as a syntax error
        
        // Find the '/' character
        let slash_pos = source.find('/').unwrap();
        lexer.set_pos(slash_pos);
        lexer.set_last_token_pos(if slash_pos > 0 { slash_pos - 1 } else { 0 });
        
        // This should succeed as a regex
        let result = lexer.handle_slash();
        prop_assert!(result.is_ok(), "Failed to parse regex context: {:?}", source);
        
        // After handling, position should have advanced past the regex
        prop_assert!(lexer.get_pos() > slash_pos, "Position did not advance after regex");
    }

    #[test]
    fn prop_division_context_detection(source in arb_division_context()) {
        let mut lexer = Lexer::new(&source);
        
        // Find the '/' character
        let slash_pos = source.find('/').unwrap();
        lexer.set_pos(slash_pos);
        
        // Set last_token_pos to point to a character before the slash
        // that should indicate division context
        if slash_pos > 0 {
            let mut pos = slash_pos - 1;
            // Skip whitespace to find the actual token
            while pos > 0 && (source.as_bytes()[pos] == b' ' || source.as_bytes()[pos] == b'\t') {
                pos -= 1;
            }
            lexer.set_last_token_pos(pos);
        }
        
        // This should be treated as division
        let result = lexer.handle_slash();
        prop_assert!(result.is_ok(), "Failed to parse division context: {:?}", source);
        
        // After handling division, position should advance by 1
        prop_assert_eq!(lexer.get_pos(), slash_pos + 1, "Position should advance by 1 for division");
    }

    #[test]
    fn prop_expression_punctuator_precedes_regex(punct in arb_expression_punctuator()) {
        let source = format!("{}/test/", punct);
        let mut lexer = Lexer::new(&source);
        
        let slash_pos = source.find('/').unwrap();
        lexer.set_pos(slash_pos);
        lexer.set_last_token_pos(slash_pos - 1);
        
        let result = lexer.handle_slash();
        prop_assert!(result.is_ok());
        prop_assert!(lexer.get_pos() > slash_pos);
    }

    #[test]
    fn prop_expression_keyword_precedes_regex(keyword in arb_expression_keyword()) {
        let source = format!("{} /test/", keyword);
        let mut lexer = Lexer::new(&source);
        
        let slash_pos = source.find('/').unwrap();
        lexer.set_pos(slash_pos);
        // Point to the last character of the keyword
        lexer.set_last_token_pos(keyword.len() - 1);
        
        let result = lexer.handle_slash();
        prop_assert!(result.is_ok());
        prop_assert!(lexer.get_pos() > slash_pos);
    }
}
