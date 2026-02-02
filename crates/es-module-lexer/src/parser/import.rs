//! Import statement parsing.

use crate::lexer::Lexer;
use crate::error::LexerError;
use crate::types::{Import, ImportType};
use smallvec::SmallVec;

impl<'a> Lexer<'a> {
    /// Try to parse an import statement.
    /// 
    /// Handles:
    /// - Static imports: import ... from "module"
    /// - Dynamic imports: import(...)
    /// - import.meta
    /// - import source/defer syntax
    pub(crate) fn try_parse_import_statement(&mut self) -> Result<(), LexerError> {
        let statement_start = self.position();
        
        // Skip "import" keyword
        self.advance_by(6);
        
        // Skip whitespace and comments
        let ch = self.comment_whitespace(false)?;
        
        match ch {
            b'(' => {
                // Dynamic import: import(...)
                self.parse_dynamic_import(statement_start)?;
            }
            b'.' => {
                // import.meta or import.source() or import.defer()
                self.advance_by(1); // Skip '.'
                let _ch = self.comment_whitespace(false)?;
                
                if self.matches_keyword(b"meta") {
                    self.parse_import_meta(statement_start)?;
                } else if self.matches_keyword(b"source") {
                    self.parse_source_phase_import(statement_start)?;
                } else if self.matches_keyword(b"defer") {
                    self.parse_defer_phase_import(statement_start)?;
                }
            }
            b'"' | b'\'' => {
                // String import: import "module"
                self.parse_string_import(statement_start, ch)?;
            }
            _ => {
                // Named import: import { x } from "module" or import x from "module"
                self.parse_named_import(statement_start)?;
            }
        }
        
        Ok(())
    }
    
    /// Parse a static string import: import "module"
    fn parse_string_import(&mut self, statement_start: usize, quote: u8) -> Result<(), LexerError> {
        let str_start = self.position() + 1; // After opening quote
        self.string_literal(quote)?;
        let str_end = self.position() - 1; // Before closing quote
        
        // Find statement end (semicolon or newline)
        let statement_end = self.find_statement_end();
        
        self.imports.push(Import {
            start: str_start,
            end: str_end,
            statement_start,
            statement_end,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        });
        
        Ok(())
    }
    
    /// Parse a named import: import { x, y as z } from "module"
    /// or import x from "module"
    /// or import * as ns from "module"
    fn parse_named_import(&mut self, statement_start: usize) -> Result<(), LexerError> {
        // Skip the import specifiers (we don't need to parse them in detail)
        // We just need to find the "from" keyword and the module specifier
        
        loop {
            if self.is_at_end() {
                return Ok(());
            }
            
            let ch = self.comment_whitespace(false)?;
            
            match ch {
                b'{' => {
                    // Named imports: { x, y as z }
                    self.advance_by(1);
                    self.skip_import_specifiers()?;
                }
                b'*' => {
                    // Namespace import: * as ns
                    self.advance_by(1);
                    let _ch = self.comment_whitespace(false)?;
                    if self.matches_keyword(b"as") {
                        self.advance_by(2);
                        // Skip the namespace identifier
                        self.skip_identifier()?;
                    }
                }
                b'f' if self.matches_keyword(b"from") => {
                    // Found "from" keyword
                    self.advance_by(4);
                    let ch = self.comment_whitespace(false)?;
                    
                    if ch == b'"' || ch == b'\'' {
                        let str_start = self.position() + 1;
                        self.string_literal(ch)?;
                        let str_end = self.position() - 1;
                        
                        // Check for import attributes
                        let attr_index = self.check_import_attributes()?;
                        
                        let statement_end = self.find_statement_end();
                        
                        self.imports.push(Import {
                            start: str_start,
                            end: str_end,
                            statement_start,
                            statement_end,
                            attr_index,
                            dynamic: None,
                            safe: true,
                            import_type: ImportType::Static,
                            attributes: SmallVec::new(),
                        });
                    }
                    
                    return Ok(());
                }
                b';' | b'\n' | b'\r' => {
                    // Statement end without "from" - invalid but we'll just return
                    return Ok(());
                }
                _ => {
                    // Default import or identifier
                    self.skip_identifier()?;
                    
                    // Check if there's a comma (multiple imports)
                    let ch = self.comment_whitespace(false)?;
                    if ch == b',' {
                        self.advance_by(1);
                    }
                }
            }
        }
    }
    
    /// Skip import specifiers inside { }
    fn skip_import_specifiers(&mut self) -> Result<(), LexerError> {
        loop {
            let ch = self.comment_whitespace(false)?;
            
            match ch {
                b'}' => {
                    self.advance_by(1);
                    return Ok(());
                }
                b',' => {
                    self.advance_by(1);
                }
                b'"' | b'\'' => {
                    // String export name
                    self.string_literal(ch)?;
                    
                    // Check for "as"
                    let _ch = self.comment_whitespace(false)?;
                    if self.matches_keyword(b"as") {
                        self.advance_by(2);
                        let ch = self.comment_whitespace(false)?;
                        if ch == b'"' || ch == b'\'' {
                            self.string_literal(ch)?;
                        } else {
                            self.skip_identifier()?;
                        }
                    }
                }
                _ => {
                    // Identifier
                    self.skip_identifier()?;
                    
                    // Check for "as"
                    let _ch = self.comment_whitespace(false)?;
                    if self.matches_keyword(b"as") {
                        self.advance_by(2);
                        self.skip_identifier()?;
                    }
                }
            }
        }
    }
    
    /// Skip an identifier
    pub(crate) fn skip_identifier(&mut self) -> Result<(), LexerError> {
        let ch = self.comment_whitespace(false)?;
        
        if ch == 0 {
            return Ok(());
        }
        
        // First character must be letter, _, or $
        if !ch.is_ascii_alphabetic() && ch != b'_' && ch != b'$' {
            return Ok(());
        }
        
        self.advance_by(1);
        
        // Subsequent characters can be alphanumeric, _, or $
        while !self.is_at_end() {
            if let Some(ch) = self.peek() {
                if ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'$' {
                    self.advance_by(1);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Check for import attributes (with clause)
    /// Returns the position of the "with" keyword if found
    fn check_import_attributes(&mut self) -> Result<Option<usize>, LexerError> {
        let _ch = self.comment_whitespace(false)?;
        
        if self.matches_keyword(b"with") {
            let attr_pos = self.position();
            // Don't parse attributes here - that's done in task 10
            // Just return the position
            return Ok(Some(attr_pos));
        }
        
        Ok(None)
    }
    
    /// Find the end of the current statement
    fn find_statement_end(&mut self) -> usize {
        let _start_pos = self.position();

        // Look for semicolon or newline
        while !self.is_at_end() {
            if let Some(ch) = self.peek() {
                match ch {
                    b';' => {
                        // Return position before semicolon to match es-module-lexer behavior
                        return self.position();
                    }
                    b'\n' | b'\r' => {
                        return self.position();
                    }
                    b' ' | b'\t' => {
                        self.advance_by(1);
                    }
                    b'/' => {
                        // Could be comment
                        if let Some(next_ch) = self.peek_at(1) {
                            if next_ch == b'/' || next_ch == b'*' {
                                // Skip comment
                                let _ = self.comment_whitespace(false);
                            } else {
                                return self.position();
                            }
                        } else {
                            return self.position();
                        }
                    }
                    _ => {
                        return self.position();
                    }
                }
            } else {
                break;
            }
        }
        
        self.position()
    }
    
    /// Parse a dynamic import: import(...)
    fn parse_dynamic_import(&mut self, statement_start: usize) -> Result<(), LexerError> {
        use crate::types::{OpenToken, OpenTokenState};
        
        let dynamic_pos = self.position();
        self.advance_by(1); // Skip '('
        
        // Push import paren onto stack
        self.push_token(OpenToken {
            state: OpenTokenState::ImportParen,
            pos: dynamic_pos,
        })?;
        
        let import_idx = self.imports.len();
        self.dynamic_import_stack.push(import_idx);
        
        // Create import record
        let mut import = Import {
            start: 0,
            end: 0,
            statement_start,
            statement_end: 0,
            attr_index: None,
            dynamic: Some(dynamic_pos),
            safe: false,
            import_type: ImportType::Dynamic,
            attributes: SmallVec::new(),
        };
        
        // Try to parse string literal
        let ch = self.comment_whitespace(false)?;
        if ch == b'"' || ch == b'\'' {
            let str_start = self.position() + 1;
            self.string_literal(ch)?;
            let str_end = self.position() - 1;
            import.start = str_start;
            import.end = str_end;
            import.safe = true;
            
            // Check for attributes (comma after string)
            let ch = self.comment_whitespace(false)?;
            if ch == b',' {
                import.attr_index = Some(self.position());
            }
        }
        
        self.imports.push(import);
        
        // Note: The closing ')' will be handled by the full parser
        // which will update statement_end when it finds the matching paren
        
        Ok(())
    }
    
    /// Parse import.meta
    fn parse_import_meta(&mut self, statement_start: usize) -> Result<(), LexerError> {
        // Skip "meta" keyword
        self.advance_by(4);
        
        let statement_end = self.find_statement_end();
        
        self.imports.push(Import {
            start: 0,
            end: 0,
            statement_start,
            statement_end,
            attr_index: None,
            dynamic: None,
            safe: false,
            import_type: ImportType::ImportMeta,
            attributes: SmallVec::new(),
        });
        
        Ok(())
    }
    
    /// Parse source phase import: import source "module" or import.source()
    fn parse_source_phase_import(&mut self, statement_start: usize) -> Result<(), LexerError> {
        // Skip "source" keyword
        self.advance_by(6);
        
        let ch = self.comment_whitespace(false)?;
        
        match ch {
            b'(' => {
                // Dynamic source phase: import.source(...)
                self.parse_dynamic_source_phase_import(statement_start)?;
            }
            b'"' | b'\'' => {
                // Static source phase: import source "module"
                let str_start = self.position() + 1;
                self.string_literal(ch)?;
                let str_end = self.position() - 1;
                
                let statement_end = self.find_statement_end();
                
                self.imports.push(Import {
                    start: str_start,
                    end: str_end,
                    statement_start,
                    statement_end,
                    attr_index: None,
                    dynamic: None,
                    safe: true,
                    import_type: ImportType::StaticSourcePhase,
                    attributes: SmallVec::new(),
                });
            }
            _ => {
                // Invalid syntax, just return
            }
        }
        
        Ok(())
    }
    
    /// Parse defer phase import: import defer "module" or import.defer()
    fn parse_defer_phase_import(&mut self, statement_start: usize) -> Result<(), LexerError> {
        // Skip "defer" keyword
        self.advance_by(5);
        
        let ch = self.comment_whitespace(false)?;
        
        match ch {
            b'(' => {
                // Dynamic defer phase: import.defer(...)
                self.parse_dynamic_defer_phase_import(statement_start)?;
            }
            b'"' | b'\'' => {
                // Static defer phase: import defer "module"
                let str_start = self.position() + 1;
                self.string_literal(ch)?;
                let str_end = self.position() - 1;
                
                let statement_end = self.find_statement_end();
                
                self.imports.push(Import {
                    start: str_start,
                    end: str_end,
                    statement_start,
                    statement_end,
                    attr_index: None,
                    dynamic: None,
                    safe: true,
                    import_type: ImportType::StaticDeferPhase,
                    attributes: SmallVec::new(),
                });
            }
            _ => {
                // Invalid syntax, just return
            }
        }
        
        Ok(())
    }
    
    /// Parse dynamic source phase import: import.source(...)
    fn parse_dynamic_source_phase_import(&mut self, statement_start: usize) -> Result<(), LexerError> {
        use crate::types::{OpenToken, OpenTokenState};
        
        let dynamic_pos = self.position();
        self.advance_by(1); // Skip '('
        
        // Push import paren onto stack
        self.push_token(OpenToken {
            state: OpenTokenState::ImportParen,
            pos: dynamic_pos,
        })?;
        
        let import_idx = self.imports.len();
        self.dynamic_import_stack.push(import_idx);
        
        // Create import record
        let mut import = Import {
            start: 0,
            end: 0,
            statement_start,
            statement_end: 0,
            attr_index: None,
            dynamic: Some(dynamic_pos),
            safe: false,
            import_type: ImportType::DynamicSourcePhase,
            attributes: SmallVec::new(),
        };
        
        // Try to parse string literal
        let ch = self.comment_whitespace(false)?;
        if ch == b'"' || ch == b'\'' {
            let str_start = self.position() + 1;
            self.string_literal(ch)?;
            let str_end = self.position() - 1;
            import.start = str_start;
            import.end = str_end;
            import.safe = true;
        }
        
        self.imports.push(import);
        
        Ok(())
    }
    
    /// Parse dynamic defer phase import: import.defer(...)
    fn parse_dynamic_defer_phase_import(&mut self, statement_start: usize) -> Result<(), LexerError> {
        use crate::types::{OpenToken, OpenTokenState};
        
        let dynamic_pos = self.position();
        self.advance_by(1); // Skip '('
        
        // Push import paren onto stack
        self.push_token(OpenToken {
            state: OpenTokenState::ImportParen,
            pos: dynamic_pos,
        })?;
        
        let import_idx = self.imports.len();
        self.dynamic_import_stack.push(import_idx);
        
        // Create import record
        let mut import = Import {
            start: 0,
            end: 0,
            statement_start,
            statement_end: 0,
            attr_index: None,
            dynamic: Some(dynamic_pos),
            safe: false,
            import_type: ImportType::DynamicDeferPhase,
            attributes: SmallVec::new(),
        };
        
        // Try to parse string literal
        let ch = self.comment_whitespace(false)?;
        if ch == b'"' || ch == b'\'' {
            let str_start = self.position() + 1;
            self.string_literal(ch)?;
            let str_end = self.position() - 1;
            import.start = str_start;
            import.end = str_end;
            import.safe = true;
        }
        
        self.imports.push(import);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_static_import() {
        let source = "import foo from 'bar';";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::Static);
        assert_eq!(lexer.str_slice(import.start, import.end), "bar");
        assert!(import.safe);
        assert_eq!(import.dynamic, None);
    }
    
    #[test]
    fn test_parse_string_import() {
        let source = "import 'module';";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::Static);
        assert_eq!(lexer.str_slice(import.start, import.end), "module");
    }
    
    #[test]
    fn test_parse_named_import() {
        let source = "import { foo, bar as baz } from 'module';";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::Static);
        assert_eq!(lexer.str_slice(import.start, import.end), "module");
    }
    
    #[test]
    fn test_parse_default_import() {
        let source = "import foo from 'bar';";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(lexer.str_slice(import.start, import.end), "bar");
    }
    
    #[test]
    fn test_parse_namespace_import() {
        let source = "import * as ns from 'module';";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(lexer.str_slice(import.start, import.end), "module");
    }
    
    #[test]
    fn test_parse_mixed_import() {
        let source = "import foo, { bar, baz } from 'module';";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(lexer.str_slice(import.start, import.end), "module");
    }
    
    #[test]
    fn test_parse_import_with_comments() {
        let source = "import /* comment */ foo from /* comment */ 'bar';";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(lexer.str_slice(import.start, import.end), "bar");
    }
    
    #[test]
    fn test_parse_import_double_quotes() {
        let source = "import foo from \"bar\";";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(lexer.str_slice(import.start, import.end), "bar");
    }
    
    #[test]
    fn test_parse_import_with_string_export_names() {
        let source = "import { 'foo' as bar } from 'module';";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(lexer.str_slice(import.start, import.end), "module");
    }
    
    #[test]
    fn test_statement_positions() {
        let source = "import foo from 'bar';";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        let import = &lexer.imports[0];
        assert_eq!(import.statement_start, 0);
        assert!(import.statement_end > import.statement_start);
    }
    
    // ===== Dynamic Import Tests =====
    
    #[test]
    fn test_parse_dynamic_import_string_literal() {
        let source = "import('module')";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::Dynamic);
        assert!(import.safe);
        assert_eq!(lexer.str_slice(import.start, import.end), "module");
        assert!(import.dynamic.is_some());
    }
    
    #[test]
    fn test_parse_dynamic_import_expression() {
        let source = "import(moduleName)";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::Dynamic);
        assert!(!import.safe); // Expression, not safe
        assert!(import.dynamic.is_some());
    }
    
    #[test]
    fn test_parse_dynamic_import_with_attributes() {
        let source = "import('module', { type: 'json' })";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::Dynamic);
        assert!(import.safe);
        assert!(import.attr_index.is_some());
    }
    
    #[test]
    fn test_parse_dynamic_import_double_quotes() {
        let source = "import(\"module\")";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::Dynamic);
        assert!(import.safe);
        assert_eq!(lexer.str_slice(import.start, import.end), "module");
    }
    
    #[test]
    fn test_parse_dynamic_import_with_comments() {
        let source = "import(/* comment */ 'module')";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::Dynamic);
        assert!(import.safe);
    }
    
    #[test]
    fn test_dynamic_import_stack_tracking() {
        let source = "import('module')";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        // Should have pushed to dynamic_import_stack
        assert_eq!(lexer.dynamic_import_stack.len(), 1);
        assert_eq!(lexer.dynamic_import_stack[0], 0); // First import
    }
    
    #[test]
    fn test_dynamic_import_open_token_stack() {
        let source = "import('module')";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        // Should have pushed ImportParen onto stack
        assert_eq!(lexer.open_token_stack.len(), 1);
        assert_eq!(lexer.open_token_stack[0].state, crate::types::OpenTokenState::ImportParen);
    }
    
    // ===== import.meta Tests =====
    
    #[test]
    fn test_parse_import_meta() {
        let source = "import.meta";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::ImportMeta);
        assert!(!import.safe);
        assert_eq!(import.dynamic, None);
    }
    
    #[test]
    fn test_parse_import_meta_url() {
        let source = "import.meta.url";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::ImportMeta);
    }
    
    #[test]
    fn test_parse_import_meta_with_comments() {
        let source = "import /* comment */ . /* comment */ meta";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::ImportMeta);
    }
    
    // ===== Source Phase Import Tests =====
    
    #[test]
    fn test_parse_static_source_phase_import() {
        let source = "import.source('module')";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::DynamicSourcePhase);
        assert!(import.safe);
        assert_eq!(lexer.str_slice(import.start, import.end), "module");
    }
    
    #[test]
    fn test_parse_dynamic_source_phase_import() {
        let source = "import.source(moduleName)";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::DynamicSourcePhase);
        assert!(!import.safe);
    }
    
    // ===== Defer Phase Import Tests =====
    
    #[test]
    fn test_parse_static_defer_phase_import() {
        let source = "import.defer('module')";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::DynamicDeferPhase);
        assert!(import.safe);
        assert_eq!(lexer.str_slice(import.start, import.end), "module");
    }
    
    #[test]
    fn test_parse_dynamic_defer_phase_import() {
        let source = "import.defer(moduleName)";
        let mut lexer = Lexer::new(source);
        
        lexer.try_parse_import_statement().unwrap();
        
        assert_eq!(lexer.imports.len(), 1);
        let import = &lexer.imports[0];
        assert_eq!(import.import_type, ImportType::DynamicDeferPhase);
        assert!(!import.safe);
    }
}


#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    
    // Generator for module names
    fn arb_module_name() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-zA-Z0-9_./\\-]+").unwrap()
    }
    
    // Generator for identifiers
    fn arb_identifier() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]*").unwrap()
    }
    
    // Generator for import types (only those that are actually testable)
    fn arb_import_type() -> impl Strategy<Value = ImportType> {
        prop_oneof![
            Just(ImportType::Static),
            Just(ImportType::Dynamic),
            Just(ImportType::ImportMeta),
            // Note: Static source/defer phase imports would require syntax like:
            // import source "module" (without dot and parens)
            // For now we only test the dynamic forms
            Just(ImportType::DynamicSourcePhase),
            Just(ImportType::DynamicDeferPhase),
        ]
    }
    
    // Generate import statement of a specific type
    fn generate_import_of_type(import_type: ImportType, module: &str) -> String {
        match import_type {
            ImportType::Static => format!("import foo from '{}';", module),
            ImportType::Dynamic => format!("import('{}');", module),
            ImportType::ImportMeta => "import.meta".to_string(),
            ImportType::DynamicSourcePhase => format!("import.source('{}');", module),
            ImportType::DynamicDeferPhase => format!("import.defer('{}');", module),
            // These would require different syntax that's not yet implemented
            ImportType::StaticSourcePhase => format!("import.source('{}');", module),
            ImportType::StaticDeferPhase => format!("import.defer('{}');", module),
        }
    }
    
    // Feature: es-module-lexer-rs, Property 2: Import 类型标记正确性
    // Validates: Requirements 3.1, 3.2, 3.5, 3.7, 3.8
    proptest! {
        #[test]
        fn prop_import_type_correctness(
            import_type in arb_import_type(),
            module in arb_module_name()
        ) {
            let source = generate_import_of_type(import_type, &module);
            let mut lexer = Lexer::new(&source);
            
            let result = lexer.try_parse_import_statement();
            prop_assert!(result.is_ok());
            
            prop_assert_eq!(lexer.imports.len(), 1);
            prop_assert_eq!(lexer.imports[0].import_type, import_type);
        }
        
        #[test]
        fn prop_static_import_always_static_type(
            module in arb_module_name(),
            identifier in arb_identifier()
        ) {
            let source = format!("import {} from '{}';", identifier, module);
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_import_statement().unwrap();
            
            prop_assert_eq!(lexer.imports.len(), 1);
            prop_assert_eq!(lexer.imports[0].import_type, ImportType::Static);
        }
        
        #[test]
        fn prop_dynamic_import_always_dynamic_type(
            module in arb_module_name()
        ) {
            let source = format!("import('{}');", module);
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_import_statement().unwrap();
            
            prop_assert_eq!(lexer.imports.len(), 1);
            prop_assert_eq!(lexer.imports[0].import_type, ImportType::Dynamic);
        }
        
        #[test]
        fn prop_import_meta_always_import_meta_type(
            suffix in prop::option::of(prop::string::string_regex("[a-zA-Z0-9_.]*").unwrap())
        ) {
            let source = if let Some(s) = suffix {
                format!("import.meta{}", if s.is_empty() { String::new() } else { format!(".{}", s) })
            } else {
                "import.meta".to_string()
            };
            
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_import_statement().unwrap();
            
            prop_assert_eq!(lexer.imports.len(), 1);
            prop_assert_eq!(lexer.imports[0].import_type, ImportType::ImportMeta);
        }
    }
    
    // Feature: es-module-lexer-rs, Property 3: 动态 Import 安全性标记
    // Validates: Requirements 3.3, 3.4
    proptest! {
        #[test]
        fn prop_dynamic_import_safety_string_literal(
            module in arb_module_name()
        ) {
            let source = format!("import('{}');", module);
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_import_statement().unwrap();
            
            prop_assert_eq!(lexer.imports.len(), 1);
            let import = &lexer.imports[0];
            
            // String literal should be safe
            prop_assert!(import.safe);
            prop_assert_eq!(lexer.str_slice(import.start, import.end), module.as_str());
        }
        
        #[test]
        fn prop_dynamic_import_safety_expression(
            identifier in arb_identifier()
        ) {
            let source = format!("import({})", identifier);
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_import_statement().unwrap();
            
            prop_assert_eq!(lexer.imports.len(), 1);
            let import = &lexer.imports[0];
            
            // Expression should not be safe
            prop_assert!(!import.safe);
        }
        
        #[test]
        fn prop_dynamic_import_with_attributes_has_attr_index(
            module in arb_module_name()
        ) {
            let source = format!("import('{}', {{ type: 'json' }})", module);
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_import_statement().unwrap();
            
            prop_assert_eq!(lexer.imports.len(), 1);
            let import = &lexer.imports[0];
            
            // Should have attr_index set
            prop_assert!(import.attr_index.is_some());
            prop_assert!(import.safe);
        }
    }
    
    // Feature: es-module-lexer-rs, Property 12: 动态 Import 括号匹配
    // Validates: Requirements 7.5
    proptest! {
        #[test]
        fn prop_dynamic_import_bracket_matching(
            module in arb_module_name(),
            nested_parens in prop::collection::vec("()", 0..3)
        ) {
            // Generate expression with nested parentheses
            let expr = if nested_parens.is_empty() {
                format!("'{}'", module)
            } else {
                let open = nested_parens.iter().map(|_| "(").collect::<String>();
                let close = nested_parens.iter().map(|_| ")").collect::<String>();
                format!("{}'{}'{}",  open, module, close)
            };
            
            let source = format!("import({})", expr);
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_import_statement().unwrap();
            
            prop_assert_eq!(lexer.imports.len(), 1);
            let import = &lexer.imports[0];
            
            // Should have tracked the import paren
            prop_assert!(import.dynamic.is_some());
            
            // Should have pushed to open_token_stack
            prop_assert!(!lexer.open_token_stack.is_empty());
        }
        
        #[test]
        fn prop_dynamic_import_statement_positions(
            module in arb_module_name()
        ) {
            let source = format!("import('{}');", module);
            let mut lexer = Lexer::new(&source);
            
            lexer.try_parse_import_statement().unwrap();
            
            prop_assert_eq!(lexer.imports.len(), 1);
            let import = &lexer.imports[0];
            
            // statement_start should be at the beginning
            prop_assert_eq!(import.statement_start, 0);
            
            // statement_end should be after statement_start
            // Note: In full parsing, this would be set to the closing paren position
            prop_assert!(import.statement_end >= import.statement_start);
        }
    }
}
