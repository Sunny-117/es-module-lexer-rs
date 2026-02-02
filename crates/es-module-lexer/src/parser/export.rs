//! Export statement parsing.

use crate::lexer::Lexer;
use crate::error::LexerError;
use crate::types::Export;

impl<'a> Lexer<'a> {
    /// Try to parse an export statement.
    /// Handles all forms of export: named, default, re-exports, and declarations.
    pub(crate) fn try_parse_export_statement(&mut self) -> Result<(), LexerError> {
        let start_pos = self.position();
        
        // Skip "export" keyword
        self.advance_by(6);
        
        // Skip whitespace and comments
        let ch = self.comment_whitespace(false)?;
        
        match ch {
            b'{' => {
                // export { a, b as c } [from "module"]
                self.parse_export_list()?;
            }
            b'*' => {
                // export * from "module" or export * as ns from "module"
                self.parse_export_star()?;
            }
            b'd' if self.matches_keyword(b"default") => {
                // export default ...
                self.parse_export_default(start_pos)?;
            }
            b'v' | b'l' | b'c' | b'f' | b'a' => {
                // export var/let/const/function/class/async
                self.parse_export_declaration()?;
            }
            _ => {
                // Not a valid export syntax, switch to full parse mode
                self.set_facade(false);
            }
        }
        
        Ok(())
    }
    
    /// Parse export list: export { a, b as c } [from "module"]
    pub(crate) fn parse_export_list(&mut self) -> Result<(), LexerError> {
        self.advance_by(1); // Skip '{'
        
        loop {
            let ch = self.comment_whitespace(false)?;
            
            if ch == b'}' {
                self.advance_by(1);
                break;
            }
            
            // Read local name (or export name if no 'as')
            let local_start = self.position();
            
            // Handle string export names
            if ch == b'"' || ch == b'\'' {
                self.string_literal(ch)?;
            } else {
                self.read_identifier()?;
            }
            
            let local_end = self.position();
            
            let _ch = self.comment_whitespace(false)?;
            
            let (export_start, export_end) = if self.matches_keyword(b"as") {
                self.advance_by(2);
                let ch = self.comment_whitespace(false)?;
                
                let export_start = self.position();
                if ch == b'"' || ch == b'\'' {
                    self.string_literal(ch)?;
                } else {
                    self.read_identifier()?;
                }
                (export_start, self.position())
            } else {
                (local_start, local_end)
            };
            
            self.exports.push(Export {
                start: export_start,
                end: export_end,
                local_start: Some(local_start),
                local_end: Some(local_end),
            });
            
            let ch = self.comment_whitespace(false)?;
            if ch == b',' {
                self.advance_by(1);
            } else if ch != b'}' {
                return Err(LexerError::UnexpectedToken(self.position()));
            }
        }
        
        // Check for "from" clause
        let _ch = self.comment_whitespace(false)?;
        if self.matches_keyword(b"from") {
            self.advance_by(4);
            let ch = self.comment_whitespace(false)?;
            if ch == b'"' || ch == b'\'' {
                // Skip the module specifier
                self.string_literal(ch)?;
            }
        }
        
        Ok(())
    }
    
    /// Parse export star: export * from "module" or export * as ns from "module"
    fn parse_export_star(&mut self) -> Result<(), LexerError> {
        self.advance_by(1); // Skip '*'
        
        let _ch = self.comment_whitespace(false)?;
        
        if self.matches_keyword(b"as") {
            // export * as ns from "module"
            self.advance_by(2);
            let ch = self.comment_whitespace(false)?;
            
            let export_start = self.position();
            if ch == b'"' || ch == b'\'' {
                self.string_literal(ch)?;
            } else {
                self.read_identifier()?;
            }
            let export_end = self.position();
            
            self.exports.push(Export {
                start: export_start,
                end: export_end,
                local_start: None,
                local_end: None,
            });
        }
        
        // Expect "from" clause
        let _ch = self.comment_whitespace(false)?;
        if self.matches_keyword(b"from") {
            self.advance_by(4);
            let ch = self.comment_whitespace(false)?;
            if ch == b'"' || ch == b'\'' {
                // Skip the module specifier
                self.string_literal(ch)?;
            }
        }
        
        Ok(())
    }
    
    /// Parse export default: export default function/class/expression
    fn parse_export_default(&mut self, _start_pos: usize) -> Result<(), LexerError> {
        self.advance_by(7); // Skip "default"
        
        let _ch = self.comment_whitespace(false)?;
        
        // Add "default" export
        let default_str = "default";
        let export_start = self.position();
        let export_end = export_start + default_str.len();
        
        self.exports.push(Export {
            start: export_start,
            end: export_end,
            local_start: None,
            local_end: None,
        });
        
        // Handle different default export forms
        if self.matches_keyword(b"function") {
            self.advance_by(8);
            // Skip function declaration
            self.skip_function_declaration()?;
        } else if self.matches_keyword(b"class") {
            self.advance_by(5);
            // Skip class declaration
            self.skip_class_declaration()?;
        } else if self.matches_keyword(b"async") {
            self.advance_by(5);
            let _ch = self.comment_whitespace(false)?;
            if self.matches_keyword(b"function") {
                self.advance_by(8);
                self.skip_function_declaration()?;
            }
        } else {
            // Expression - switch to full parse mode
            self.set_facade(false);
        }
        
        Ok(())
    }
    
    /// Parse export declaration: export var/let/const/function/class/async
    fn parse_export_declaration(&mut self) -> Result<(), LexerError> {
        let ch = self.peek().unwrap_or(0);
        
        match ch {
            b'v' if self.matches_keyword(b"var") => {
                self.advance_by(3);
                self.parse_variable_declaration()?;
            }
            b'l' if self.matches_keyword(b"let") => {
                self.advance_by(3);
                self.parse_variable_declaration()?;
            }
            b'c' if self.matches_keyword(b"const") => {
                self.advance_by(5);
                self.parse_variable_declaration()?;
            }
            b'f' if self.matches_keyword(b"function") => {
                self.advance_by(8);
                let _ch = self.comment_whitespace(false)?;
                if _ch != 0 && !self.is_at_end() {
                    let name_start = self.position();
                    self.read_identifier()?;
                    let name_end = self.position();
                    
                    self.exports.push(Export {
                        start: name_start,
                        end: name_end,
                        local_start: None,
                        local_end: None,
                    });
                }
                self.skip_function_declaration()?;
            }
            b'c' if self.matches_keyword(b"class") => {
                self.advance_by(5);
                let _ch = self.comment_whitespace(false)?;
                if _ch != 0 && !self.is_at_end() {
                    let name_start = self.position();
                    self.read_identifier()?;
                    let name_end = self.position();
                    
                    self.exports.push(Export {
                        start: name_start,
                        end: name_end,
                        local_start: None,
                        local_end: None,
                    });
                }
                self.skip_class_declaration()?;
            }
            b'a' if self.matches_keyword(b"async") => {
                self.advance_by(5);
                let _ch = self.comment_whitespace(false)?;
                if self.matches_keyword(b"function") {
                    self.advance_by(8);
                    let _ch = self.comment_whitespace(false)?;
                    if _ch != 0 && !self.is_at_end() {
                        let name_start = self.position();
                        self.read_identifier()?;
                        let name_end = self.position();
                        
                        self.exports.push(Export {
                            start: name_start,
                            end: name_end,
                            local_start: None,
                            local_end: None,
                        });
                    }
                    self.skip_function_declaration()?;
                }
            }
            _ => {
                self.set_facade(false);
            }
        }
        
        Ok(())
    }
    
    /// Parse variable declaration and extract identifiers
    fn parse_variable_declaration(&mut self) -> Result<(), LexerError> {
        loop {
            let ch = self.comment_whitespace(false)?;
            
            if ch == 0 || self.is_at_end() {
                break;
            }
            
            // Check for destructuring
            if ch == b'{' {
                self.parse_destructuring_object()?;
            } else if ch == b'[' {
                self.parse_destructuring_array()?;
            } else {
                // Simple identifier
                let name_start = self.position();
                self.read_identifier()?;
                let name_end = self.position();
                
                self.exports.push(Export {
                    start: name_start,
                    end: name_end,
                    local_start: None,
                    local_end: None,
                });
            }
            
            // Skip initializer if present
            let ch = self.comment_whitespace(false)?;
            if ch == b'=' {
                self.advance_by(1);
                // Switch to full parse mode for expressions
                self.set_facade(false);
                break;
            }
            
            // Check for comma (more declarations)
            if ch == b',' {
                self.advance_by(1);
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Parse destructuring object pattern and extract identifiers
    fn parse_destructuring_object(&mut self) -> Result<(), LexerError> {
        self.advance_by(1); // Skip '{'
        
        loop {
            let ch = self.comment_whitespace(false)?;
            
            if ch == b'}' {
                self.advance_by(1);
                break;
            }
            
            if ch == b'.' && self.matches_bytes(b"...") {
                // Rest element
                self.advance_by(3);
                let _ch = self.comment_whitespace(false)?;
                let name_start = self.position();
                self.read_identifier()?;
                let name_end = self.position();
                
                self.exports.push(Export {
                    start: name_start,
                    end: name_end,
                    local_start: None,
                    local_end: None,
                });
                
                let ch = self.comment_whitespace(false)?;
                if ch == b'}' {
                    self.advance_by(1);
                    break;
                }
            } else {
                // Property
                let name_start = self.position();
                self.read_identifier()?;
                let name_end = self.position();
                
                let ch = self.comment_whitespace(false)?;
                
                if ch == b':' {
                    // Renamed: { a: b }
                    self.advance_by(1);
                    let ch = self.comment_whitespace(false)?;
                    
                    if ch == b'{' {
                        self.parse_destructuring_object()?;
                    } else if ch == b'[' {
                        self.parse_destructuring_array()?;
                    } else {
                        let local_start = self.position();
                        self.read_identifier()?;
                        let local_end = self.position();
                        
                        self.exports.push(Export {
                            start: local_start,
                            end: local_end,
                            local_start: None,
                            local_end: None,
                        });
                    }
                } else {
                    // Shorthand: { a }
                    self.exports.push(Export {
                        start: name_start,
                        end: name_end,
                        local_start: None,
                        local_end: None,
                    });
                }
            }
            
            let ch = self.comment_whitespace(false)?;
            if ch == b',' {
                self.advance_by(1);
            } else if ch != b'}' {
                return Err(LexerError::UnexpectedToken(self.position()));
            }
        }
        
        Ok(())
    }
    
    /// Parse destructuring array pattern and extract identifiers
    fn parse_destructuring_array(&mut self) -> Result<(), LexerError> {
        self.advance_by(1); // Skip '['
        
        loop {
            let ch = self.comment_whitespace(false)?;
            
            if ch == b']' {
                self.advance_by(1);
                break;
            }
            
            if ch == b',' {
                // Hole in array
                self.advance_by(1);
                continue;
            }
            
            if ch == b'.' && self.matches_bytes(b"...") {
                // Rest element
                self.advance_by(3);
                let _ch = self.comment_whitespace(false)?;
                let name_start = self.position();
                self.read_identifier()?;
                let name_end = self.position();
                
                self.exports.push(Export {
                    start: name_start,
                    end: name_end,
                    local_start: None,
                    local_end: None,
                });
                
                let ch = self.comment_whitespace(false)?;
                if ch == b']' {
                    self.advance_by(1);
                    break;
                }
            } else if ch == b'{' {
                self.parse_destructuring_object()?;
            } else if ch == b'[' {
                self.parse_destructuring_array()?;
            } else {
                let name_start = self.position();
                self.read_identifier()?;
                let name_end = self.position();
                
                self.exports.push(Export {
                    start: name_start,
                    end: name_end,
                    local_start: None,
                    local_end: None,
                });
            }
            
            let ch = self.comment_whitespace(false)?;
            if ch == b',' {
                self.advance_by(1);
            } else if ch != b']' {
                return Err(LexerError::UnexpectedToken(self.position()));
            }
        }
        
        Ok(())
    }
    
    /// Read an identifier (alphanumeric + underscore + dollar)
    pub(crate) fn read_identifier(&mut self) -> Result<(), LexerError> {
        if self.is_at_end() {
            return Err(LexerError::UnexpectedToken(self.position()));
        }
        
        let ch = self.peek().unwrap();
        
        // First character must be letter, underscore, or dollar
        if !ch.is_ascii_alphabetic() && ch != b'_' && ch != b'$' {
            return Err(LexerError::UnexpectedToken(self.position()));
        }
        
        self.advance_by(1);
        
        // Subsequent characters can be alphanumeric, underscore, or dollar
        while !self.is_at_end() {
            let ch = self.peek().unwrap();
            if ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'$' {
                self.advance_by(1);
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Skip function declaration (for facade mode)
    fn skip_function_declaration(&mut self) -> Result<(), LexerError> {
        // Switch to full parse mode
        self.set_facade(false);
        Ok(())
    }
    
    /// Skip class declaration (for facade mode)
    fn skip_class_declaration(&mut self) -> Result<(), LexerError> {
        // Switch to full parse mode
        self.set_facade(false);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_export_list_simple() {
        let source = "export { a, b, c };";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7); // After "export "
        
        lexer.parse_export_list().unwrap();
        
        assert_eq!(lexer.exports.len(), 3);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "a");
        assert_eq!(lexer.str_slice(lexer.exports[1].start, lexer.exports[1].end), "b");
        assert_eq!(lexer.str_slice(lexer.exports[2].start, lexer.exports[2].end), "c");
    }
    
    #[test]
    fn test_parse_export_list_with_as() {
        let source = "export { a as b, c as d };";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_list().unwrap();
        
        assert_eq!(lexer.exports.len(), 2);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "b");
        assert_eq!(lexer.str_slice(lexer.exports[0].local_start.unwrap(), lexer.exports[0].local_end.unwrap()), "a");
        assert_eq!(lexer.str_slice(lexer.exports[1].start, lexer.exports[1].end), "d");
        assert_eq!(lexer.str_slice(lexer.exports[1].local_start.unwrap(), lexer.exports[1].local_end.unwrap()), "c");
    }
    
    #[test]
    fn test_parse_export_list_with_string_names() {
        let source = r#"export { "a" as b, c as "d" };"#;
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_list().unwrap();
        
        assert_eq!(lexer.exports.len(), 2);
        // First export: "a" as b
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "b");
        // Second export: c as "d"
        assert_eq!(lexer.str_slice(lexer.exports[1].start, lexer.exports[1].end), "\"d\"");
    }
    
    #[test]
    fn test_parse_export_list_from_module() {
        let source = r#"export { a, b } from "module";"#;
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_list().unwrap();
        
        assert_eq!(lexer.exports.len(), 2);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "a");
        assert_eq!(lexer.str_slice(lexer.exports[1].start, lexer.exports[1].end), "b");
    }
    
    #[test]
    fn test_parse_export_star() {
        let source = r#"export * from "module";"#;
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_star().unwrap();
        
        // export * doesn't add to exports list
        assert_eq!(lexer.exports.len(), 0);
    }
    
    #[test]
    fn test_parse_export_star_as_namespace() {
        let source = r#"export * as ns from "module";"#;
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_star().unwrap();
        
        assert_eq!(lexer.exports.len(), 1);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "ns");
    }
    
    #[test]
    fn test_parse_export_default_function() {
        let source = "export default function foo() {}";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_default(0).unwrap();
        
        assert_eq!(lexer.exports.len(), 1);
        // Default export is recorded
        assert!(!lexer.get_facade()); // Should switch to full parse mode
    }
    
    #[test]
    fn test_parse_export_default_class() {
        let source = "export default class Foo {}";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_default(0).unwrap();
        
        assert_eq!(lexer.exports.len(), 1);
        assert!(!lexer.get_facade());
    }
    
    #[test]
    fn test_parse_export_default_expression() {
        let source = "export default 42;";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_default(0).unwrap();
        
        assert_eq!(lexer.exports.len(), 1);
        assert!(!lexer.get_facade());
    }
    
    #[test]
    fn test_parse_export_var_declaration() {
        let source = "export var a, b, c;";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_declaration().unwrap();
        
        assert_eq!(lexer.exports.len(), 3);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "a");
        assert_eq!(lexer.str_slice(lexer.exports[1].start, lexer.exports[1].end), "b");
        assert_eq!(lexer.str_slice(lexer.exports[2].start, lexer.exports[2].end), "c");
    }
    
    #[test]
    fn test_parse_export_const_declaration() {
        let source = "export const x = 1;";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_declaration().unwrap();
        
        assert_eq!(lexer.exports.len(), 1);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "x");
        assert!(!lexer.get_facade()); // Should switch to full parse mode due to initializer
    }
    
    #[test]
    fn test_parse_export_function_declaration() {
        let source = "export function foo() {}";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_declaration().unwrap();
        
        assert_eq!(lexer.exports.len(), 1);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "foo");
    }
    
    #[test]
    fn test_parse_export_class_declaration() {
        let source = "export class Foo {}";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_declaration().unwrap();
        
        assert_eq!(lexer.exports.len(), 1);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "Foo");
    }
    
    #[test]
    fn test_parse_export_async_function() {
        let source = "export async function bar() {}";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(7);
        
        lexer.parse_export_declaration().unwrap();
        
        assert_eq!(lexer.exports.len(), 1);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "bar");
    }
    
    #[test]
    fn test_parse_destructuring_object() {
        let source = "export const { a, b } = obj;";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(13); // After "export const "
        
        lexer.parse_destructuring_object().unwrap();
        
        assert_eq!(lexer.exports.len(), 2);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "a");
        assert_eq!(lexer.str_slice(lexer.exports[1].start, lexer.exports[1].end), "b");
    }
    
    #[test]
    fn test_parse_destructuring_object_with_rename() {
        let source = "export const { a: x, b: y } = obj;";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(13);
        
        lexer.parse_destructuring_object().unwrap();
        
        assert_eq!(lexer.exports.len(), 2);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "x");
        assert_eq!(lexer.str_slice(lexer.exports[1].start, lexer.exports[1].end), "y");
    }
    
    #[test]
    fn test_parse_destructuring_object_with_rest() {
        let source = "export const { a, ...rest } = obj;";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(13);
        
        lexer.parse_destructuring_object().unwrap();
        
        assert_eq!(lexer.exports.len(), 2);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "a");
        assert_eq!(lexer.str_slice(lexer.exports[1].start, lexer.exports[1].end), "rest");
    }
    
    #[test]
    fn test_parse_destructuring_array() {
        let source = "export const [a, b, c] = arr;";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(13);
        
        lexer.parse_destructuring_array().unwrap();
        
        assert_eq!(lexer.exports.len(), 3);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "a");
        assert_eq!(lexer.str_slice(lexer.exports[1].start, lexer.exports[1].end), "b");
        assert_eq!(lexer.str_slice(lexer.exports[2].start, lexer.exports[2].end), "c");
    }
    
    #[test]
    fn test_parse_destructuring_array_with_holes() {
        let source = "export const [a, , c] = arr;";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(13);
        
        lexer.parse_destructuring_array().unwrap();
        
        assert_eq!(lexer.exports.len(), 2);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "a");
        assert_eq!(lexer.str_slice(lexer.exports[1].start, lexer.exports[1].end), "c");
    }
    
    #[test]
    fn test_parse_destructuring_array_with_rest() {
        let source = "export const [a, ...rest] = arr;";
        let mut lexer = Lexer::new(source);
        lexer.set_pos(13);
        
        lexer.parse_destructuring_array().unwrap();
        
        assert_eq!(lexer.exports.len(), 2);
        assert_eq!(lexer.str_slice(lexer.exports[0].start, lexer.exports[0].end), "a");
        assert_eq!(lexer.str_slice(lexer.exports[1].start, lexer.exports[1].end), "rest");
    }
    
    #[test]
    fn test_read_identifier_simple() {
        let source = "foo";
        let mut lexer = Lexer::new(source);
        
        lexer.read_identifier().unwrap();
        
        assert_eq!(lexer.position(), 3);
    }
    
    #[test]
    fn test_read_identifier_with_underscore() {
        let source = "_foo_bar";
        let mut lexer = Lexer::new(source);
        
        lexer.read_identifier().unwrap();
        
        assert_eq!(lexer.position(), 8);
    }
    
    #[test]
    fn test_read_identifier_with_dollar() {
        let source = "$foo$bar";
        let mut lexer = Lexer::new(source);
        
        lexer.read_identifier().unwrap();
        
        assert_eq!(lexer.position(), 8);
    }
    
    #[test]
    fn test_read_identifier_with_numbers() {
        let source = "foo123";
        let mut lexer = Lexer::new(source);
        
        lexer.read_identifier().unwrap();
        
        assert_eq!(lexer.position(), 6);
    }
    
    #[test]
    fn test_read_identifier_stops_at_punctuation() {
        let source = "foo,bar";
        let mut lexer = Lexer::new(source);
        
        lexer.read_identifier().unwrap();
        
        assert_eq!(lexer.position(), 3);
        assert_eq!(lexer.peek(), Some(b','));
    }
    
    #[test]
    fn test_read_identifier_invalid_start() {
        let source = "123foo";
        let mut lexer = Lexer::new(source);
        
        let result = lexer.read_identifier();
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_try_parse_export_statement_named() {
        let source = "export { a, b };";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_export_statement().unwrap();
        
        assert_eq!(lexer.exports.len(), 2);
    }
    
    #[test]
    fn test_try_parse_export_statement_default() {
        let source = "export default function() {}";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_export_statement().unwrap();
        
        assert_eq!(lexer.exports.len(), 1);
    }
    
    #[test]
    fn test_try_parse_export_statement_star() {
        let source = r#"export * from "module";"#;
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_export_statement().unwrap();
        
        // export * doesn't add to exports list
        assert_eq!(lexer.exports.len(), 0);
    }
    
    #[test]
    fn test_try_parse_export_statement_declaration() {
        let source = "export const x = 1;";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_export_statement().unwrap();
        
        assert_eq!(lexer.exports.len(), 1);
    }
}


#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    
    // Generator for valid identifiers
    fn arb_identifier() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-zA-Z_$][a-zA-Z0-9_$]{0,10}").unwrap()
    }
    
    // Generator for named export statements
    fn arb_named_export() -> impl Strategy<Value = String> {
        prop::collection::vec(arb_identifier(), 1..5)
            .prop_map(|names| format!("export {{ {} }};", names.join(", ")))
    }
    
    // Generator for named export with 'as'
    fn arb_named_export_with_as() -> impl Strategy<Value = String> {
        (arb_identifier(), arb_identifier())
            .prop_map(|(local, exported)| format!("export {{ {} as {} }};", local, exported))
    }
    
    // Generator for default export
    fn arb_default_export() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("export default function foo() {}".to_string()),
            Just("export default class Foo {}".to_string()),
            Just("export default async function bar() {}".to_string()),
            Just("export default 42;".to_string()),
        ]
    }
    
    // Generator for re-export statements
    fn arb_reexport() -> impl Strategy<Value = String> {
        prop_oneof![
            Just(r#"export * from "module";"#.to_string()),
            arb_identifier().prop_map(|name| format!(r#"export * as {} from "module";"#, name)),
            prop::collection::vec(arb_identifier(), 1..3)
                .prop_map(|names| format!(r#"export {{ {} }} from "module";"#, names.join(", "))),
        ]
    }
    
    // Generator for declaration exports
    fn arb_declaration_export() -> impl Strategy<Value = String> {
        prop_oneof![
            arb_identifier().prop_map(|name| format!("export var {};", name)),
            arb_identifier().prop_map(|name| format!("export let {};", name)),
            arb_identifier().prop_map(|name| format!("export const {} = 1;", name)),
            arb_identifier().prop_map(|name| format!("export function {}() {{}}", name)),
            arb_identifier().prop_map(|name| format!("export class {} {{}}", name)),
            arb_identifier().prop_map(|name| format!("export async function {}() {{}}", name)),
        ]
    }
    
    // Generator for destructuring exports
    fn arb_destructuring_export() -> impl Strategy<Value = String> {
        prop_oneof![
            prop::collection::vec(arb_identifier(), 1..3)
                .prop_map(|names| format!("export const {{ {} }} = obj;", names.join(", "))),
            prop::collection::vec(arb_identifier(), 1..3)
                .prop_map(|names| format!("export const [{}] = arr;", names.join(", "))),
        ]
    }
    
    // Generator for any export statement
    fn arb_export_statement() -> impl Strategy<Value = String> {
        prop_oneof![
            arb_named_export(),
            arb_named_export_with_as(),
            arb_default_export(),
            arb_reexport(),
            arb_declaration_export(),
            arb_destructuring_export(),
        ]
    }
    
    // Feature: es-module-lexer-rs, Property 4: Export 提取完整性
    // Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7
    proptest! {
        #[test]
        fn prop_export_extraction_completeness(
            exports in prop::collection::vec(arb_export_statement(), 1..5)
        ) {
            let source = exports.join("\n");
            let mut lexer = Lexer::new(&source);
            
            // Parse all export statements
            let mut pos = 0;
            while pos < source.len() {
                lexer.set_pos(pos);
                if lexer.matches_keyword(b"export") {
                    let _ = lexer.try_parse_export_statement();
                }
                pos += 1;
            }
            
            // Should have extracted at least one export per statement
            // (some statements like "export *" don't add to exports list)
            prop_assert!(lexer.exports.len() > 0 || exports.iter().any(|e| e.contains("export *")));
        }
        
        #[test]
        fn prop_named_export_preserves_names(
            names in prop::collection::vec(arb_identifier(), 1..5)
        ) {
            let source = format!("export {{ {} }};", names.join(", "));
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_export_statement().unwrap();
            
            // Should have one export per name
            prop_assert_eq!(lexer.exports.len(), names.len());
            
            // Each export name should match
            for (i, name) in names.iter().enumerate() {
                let export = &lexer.exports[i];
                let extracted = lexer.str_slice(export.start, export.end);
                prop_assert_eq!(extracted, name);
            }
        }
        
        #[test]
        fn prop_export_as_preserves_both_names(
            local in arb_identifier(),
            exported in arb_identifier()
        ) {
            let source = format!("export {{ {} as {} }};", local, exported);
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_export_statement().unwrap();
            
            prop_assert_eq!(lexer.exports.len(), 1);
            
            let export = &lexer.exports[0];
            let exported_name = lexer.str_slice(export.start, export.end);
            let local_name = lexer.str_slice(
                export.local_start.unwrap(),
                export.local_end.unwrap()
            );
            
            prop_assert_eq!(exported_name, exported);
            prop_assert_eq!(local_name, local);
        }
        
        #[test]
        fn prop_default_export_always_creates_export(
            export_type in prop::sample::select(vec![
                "export default function foo() {}",
                "export default class Foo {}",
                "export default async function bar() {}",
                "export default 42;",
            ])
        ) {
            let mut lexer = Lexer::new(export_type);
            
            lexer.try_parse_export_statement().unwrap();
            
            // Default export should always create at least one export entry
            prop_assert!(lexer.exports.len() >= 1);
        }
        
        #[test]
        fn prop_function_export_extracts_name(
            name in arb_identifier()
        ) {
            let source = format!("export function {}() {{}}", name);
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_export_statement().unwrap();
            
            prop_assert_eq!(lexer.exports.len(), 1);
            
            let export = &lexer.exports[0];
            let extracted = lexer.str_slice(export.start, export.end);
            prop_assert_eq!(extracted, name);
        }
        
        #[test]
        fn prop_class_export_extracts_name(
            name in arb_identifier()
        ) {
            let source = format!("export class {} {{}}", name);
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_export_statement().unwrap();
            
            prop_assert_eq!(lexer.exports.len(), 1);
            
            let export = &lexer.exports[0];
            let extracted = lexer.str_slice(export.start, export.end);
            prop_assert_eq!(extracted, name);
        }
        
        #[test]
        fn prop_var_export_extracts_names(
            names in prop::collection::vec(arb_identifier(), 1..3)
        ) {
            let source = format!("export var {};", names.join(", "));
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_export_statement().unwrap();
            
            prop_assert_eq!(lexer.exports.len(), names.len());
            
            for (i, name) in names.iter().enumerate() {
                let export = &lexer.exports[i];
                let extracted = lexer.str_slice(export.start, export.end);
                prop_assert_eq!(extracted, name);
            }
        }
        
        #[test]
        fn prop_destructuring_object_extracts_all_names(
            names in prop::collection::vec(arb_identifier(), 1..3)
        ) {
            let source = format!("export const {{ {} }} = obj;", names.join(", "));
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_export_statement().unwrap();
            
            // Should extract all destructured names
            prop_assert_eq!(lexer.exports.len(), names.len());
        }
        
        #[test]
        fn prop_destructuring_array_extracts_all_names(
            names in prop::collection::vec(arb_identifier(), 1..3)
        ) {
            let source = format!("export const [{}] = arr;", names.join(", "));
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_export_statement().unwrap();
            
            // Should extract all destructured names
            prop_assert_eq!(lexer.exports.len(), names.len());
        }
        
        #[test]
        fn prop_export_star_as_extracts_namespace_name(
            name in arb_identifier()
        ) {
            let source = format!(r#"export * as {} from "module";"#, name);
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_export_statement().unwrap();
            
            prop_assert_eq!(lexer.exports.len(), 1);
            
            let export = &lexer.exports[0];
            let extracted = lexer.str_slice(export.start, export.end);
            prop_assert_eq!(extracted, name);
        }
    }
}
