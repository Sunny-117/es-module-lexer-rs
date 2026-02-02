//! Import attributes parsing.

use crate::lexer::Lexer;
use crate::error::LexerError;
use crate::types::Attribute;
use smallvec::SmallVec;

impl<'a> Lexer<'a> {
    /// Parse import attributes (with clause).
    /// 
    /// Parses the `with { key: "value" }` syntax for import attributes.
    /// Handles multiple key-value pairs and string escaping.
    /// 
    /// # Arguments
    /// * `import_idx` - Index of the import in the imports vector
    /// 
    /// # Returns
    /// Ok(()) if parsing succeeds, Err if syntax error
    #[allow(dead_code)]
    pub(crate) fn parse_import_attributes(&mut self, import_idx: usize) -> Result<(), LexerError> {
        // Check for "with" keyword
        let ch = self.comment_whitespace(true)?;
        
        if ch != b'w' || !self.matches_keyword(b"with") {
            return Ok(());
        }
        
        self.advance_by(4); // Skip "with"
        
        // Expect '{'
        let ch = self.comment_whitespace(true)?;
        if ch != b'{' {
            return Ok(());
        }
        
        self.advance_by(1); // Skip '{'
        
        let mut attributes = SmallVec::new();
        
        loop {
            let ch = self.comment_whitespace(true)?;
            
            if ch == b'}' {
                self.advance_by(1); // Skip '}'
                break;
            }
            
            if ch == 0 {
                // Unexpected end of file
                return Err(LexerError::UnexpectedToken(self.position()));
            }
            
            // Parse key
            let key_start = self.position();
            
            if ch == b'"' || ch == b'\'' {
                // String key
                self.string_literal(ch)?;
            } else if ch.is_ascii_alphabetic() || ch == b'_' || ch == b'$' {
                // Identifier key
                self.skip_identifier()?;
            } else {
                // Invalid key
                return Err(LexerError::UnexpectedToken(self.position()));
            }
            
            let key_end = self.position();
            
            // Expect ':'
            let ch = self.comment_whitespace(true)?;
            if ch != b':' {
                return Err(LexerError::ExpectedColon(self.position()));
            }
            self.advance_by(1); // Skip ':'
            
            // Parse value (must be a string)
            let ch = self.comment_whitespace(true)?;
            if ch != b'"' && ch != b'\'' {
                return Err(LexerError::ExpectedString(self.position()));
            }
            
            let value_start = self.position();
            self.string_literal(ch)?;
            let value_end = self.position();
            
            // Add attribute
            attributes.push(Attribute {
                key_start,
                key_end,
                value_start,
                value_end,
            });
            
            // Check for comma or closing brace
            let ch = self.comment_whitespace(true)?;
            if ch == b',' {
                self.advance_by(1); // Skip ','
            } else if ch != b'}' {
                return Err(LexerError::UnexpectedToken(self.position()));
            }
        }
        
        // Update the import with attributes
        if let Some(import) = self.imports.get_mut(import_idx) {
            import.attributes = attributes;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Import, ImportType};

    #[test]
    fn test_parse_import_attributes_simple() {
        let source = r#"with { type: "json" }"#;
        let mut lexer = Lexer::new(source);
        
        // Add a dummy import
        lexer.imports.push(Import {
            start: 0,
            end: 0,
            statement_start: 0,
            statement_end: 0,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        });
        
        assert!(lexer.parse_import_attributes(0).is_ok());
        assert_eq!(lexer.imports[0].attributes.len(), 1);
        
        let attr = &lexer.imports[0].attributes[0];
        assert_eq!(lexer.str_slice(attr.key_start, attr.key_end), "type");
        assert_eq!(lexer.str_slice(attr.value_start, attr.value_end), "\"json\"");
    }

    #[test]
    fn test_parse_import_attributes_multiple() {
        let source = r#"with { type: "json", integrity: "sha384-..." }"#;
        let mut lexer = Lexer::new(source);
        
        lexer.imports.push(Import {
            start: 0,
            end: 0,
            statement_start: 0,
            statement_end: 0,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        });
        
        assert!(lexer.parse_import_attributes(0).is_ok());
        assert_eq!(lexer.imports[0].attributes.len(), 2);
        
        let attr1 = &lexer.imports[0].attributes[0];
        assert_eq!(lexer.str_slice(attr1.key_start, attr1.key_end), "type");
        assert_eq!(lexer.str_slice(attr1.value_start, attr1.value_end), "\"json\"");
        
        let attr2 = &lexer.imports[0].attributes[1];
        assert_eq!(lexer.str_slice(attr2.key_start, attr2.key_end), "integrity");
        assert_eq!(lexer.str_slice(attr2.value_start, attr2.value_end), "\"sha384-...\"");
    }

    #[test]
    fn test_parse_import_attributes_string_key() {
        let source = r#"with { "type": "json" }"#;
        let mut lexer = Lexer::new(source);
        
        lexer.imports.push(Import {
            start: 0,
            end: 0,
            statement_start: 0,
            statement_end: 0,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        });
        
        assert!(lexer.parse_import_attributes(0).is_ok());
        assert_eq!(lexer.imports[0].attributes.len(), 1);
        
        let attr = &lexer.imports[0].attributes[0];
        assert_eq!(lexer.str_slice(attr.key_start, attr.key_end), "\"type\"");
        assert_eq!(lexer.str_slice(attr.value_start, attr.value_end), "\"json\"");
    }

    #[test]
    fn test_parse_import_attributes_with_whitespace() {
        let source = r#"with  {  type  :  "json"  ,  foo  :  "bar"  }  "#;
        let mut lexer = Lexer::new(source);
        
        lexer.imports.push(Import {
            start: 0,
            end: 0,
            statement_start: 0,
            statement_end: 0,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        });
        
        assert!(lexer.parse_import_attributes(0).is_ok());
        assert_eq!(lexer.imports[0].attributes.len(), 2);
    }

    #[test]
    fn test_parse_import_attributes_no_with_keyword() {
        let source = r#"{ type: "json" }"#;
        let mut lexer = Lexer::new(source);
        
        lexer.imports.push(Import {
            start: 0,
            end: 0,
            statement_start: 0,
            statement_end: 0,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        });
        
        assert!(lexer.parse_import_attributes(0).is_ok());
        assert_eq!(lexer.imports[0].attributes.len(), 0); // No attributes parsed
    }

    #[test]
    fn test_parse_import_attributes_empty() {
        let source = r#"with { }"#;
        let mut lexer = Lexer::new(source);
        
        lexer.imports.push(Import {
            start: 0,
            end: 0,
            statement_start: 0,
            statement_end: 0,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        });
        
        assert!(lexer.parse_import_attributes(0).is_ok());
        assert_eq!(lexer.imports[0].attributes.len(), 0);
    }

    #[test]
    fn test_parse_import_attributes_missing_colon() {
        let source = r#"with { type "json" }"#;
        let mut lexer = Lexer::new(source);
        
        lexer.imports.push(Import {
            start: 0,
            end: 0,
            statement_start: 0,
            statement_end: 0,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        });
        
        let result = lexer.parse_import_attributes(0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::ExpectedColon(_)));
    }

    #[test]
    fn test_parse_import_attributes_non_string_value() {
        let source = r#"with { type: json }"#;
        let mut lexer = Lexer::new(source);
        
        lexer.imports.push(Import {
            start: 0,
            end: 0,
            statement_start: 0,
            statement_end: 0,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        });
        
        let result = lexer.parse_import_attributes(0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LexerError::ExpectedString(_)));
    }

    #[test]
    fn test_parse_import_attributes_with_escaped_strings() {
        let source = r#"with { "ty\"pe": "js\non" }"#;
        let mut lexer = Lexer::new(source);
        
        lexer.imports.push(Import {
            start: 0,
            end: 0,
            statement_start: 0,
            statement_end: 0,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        });
        
        assert!(lexer.parse_import_attributes(0).is_ok());
        assert_eq!(lexer.imports[0].attributes.len(), 1);
    }

    #[test]
    fn test_parse_import_attributes_trailing_comma() {
        let source = r#"with { type: "json", }"#;
        let mut lexer = Lexer::new(source);
        
        lexer.imports.push(Import {
            start: 0,
            end: 0,
            statement_start: 0,
            statement_end: 0,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        });
        
        assert!(lexer.parse_import_attributes(0).is_ok());
        assert_eq!(lexer.imports[0].attributes.len(), 1);
    }
}


#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use crate::types::{Import, ImportType};

    // Generator for valid attribute keys (identifiers or strings)
    fn arb_attribute_key() -> impl Strategy<Value = String> {
        prop_oneof![
            // Identifier keys
            prop::string::string_regex("[a-zA-Z_$][a-zA-Z0-9_$]*").unwrap(),
            // String keys (simple, no escapes for now)
            prop::string::string_regex("[a-zA-Z0-9_-]+").unwrap()
                .prop_map(|s| format!("\"{}\"", s)),
        ]
    }

    // Generator for valid attribute values (must be strings)
    fn arb_attribute_value() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-zA-Z0-9_./:-]+").unwrap()
            .prop_map(|s| format!("\"{}\"", s))
    }

    // Generator for a single attribute key-value pair
    fn arb_attribute_pair() -> impl Strategy<Value = (String, String)> {
        (arb_attribute_key(), arb_attribute_value())
    }

    // Generator for import attributes with clause
    fn arb_import_attributes() -> impl Strategy<Value = (String, Vec<(String, String)>)> {
        prop::collection::vec(arb_attribute_pair(), 1..5)
            .prop_map(|pairs| {
                let pairs_str = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ");
                let source = format!("with {{ {} }}", pairs_str);
                (source, pairs)
            })
    }

    // Feature: es-module-lexer-rs, Property 5: Import Attributes 解析完整性
    // Validates: Requirements 5.1, 5.4
    proptest! {
        #[test]
        fn prop_import_attributes_parsing_completeness(
            (source, expected_pairs) in arb_import_attributes()
        ) {
            let mut lexer = Lexer::new(&source);
            
            // Add a dummy import
            lexer.imports.push(Import {
                start: 0,
                end: 0,
                statement_start: 0,
                statement_end: 0,
                attr_index: None,
                dynamic: None,
                safe: true,
                import_type: ImportType::Static,
                attributes: SmallVec::new(),
            });
            
            // Parse attributes
            let result = lexer.parse_import_attributes(0);
            prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
            
            // Verify all key-value pairs were parsed
            let import = &lexer.imports[0];
            prop_assert_eq!(
                import.attributes.len(),
                expected_pairs.len(),
                "Expected {} attributes, got {}",
                expected_pairs.len(),
                import.attributes.len()
            );
            
            // Verify each attribute
            for (i, (expected_key, expected_value)) in expected_pairs.iter().enumerate() {
                let attr = &import.attributes[i];
                let actual_key = lexer.str_slice(attr.key_start, attr.key_end);
                let actual_value = lexer.str_slice(attr.value_start, attr.value_end);
                
                prop_assert_eq!(
                    actual_key, expected_key.as_str(),
                    "Attribute {} key mismatch", i
                );
                prop_assert_eq!(
                    actual_value, expected_value.as_str(),
                    "Attribute {} value mismatch", i
                );
            }
        }

        #[test]
        fn prop_import_attributes_order_preserved(
            pairs in prop::collection::vec(arb_attribute_pair(), 2..5)
        ) {
            let pairs_str = pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            let source = format!("with {{ {} }}", pairs_str);
            
            let mut lexer = Lexer::new(&source);
            lexer.imports.push(Import {
                start: 0,
                end: 0,
                statement_start: 0,
                statement_end: 0,
                attr_index: None,
                dynamic: None,
                safe: true,
                import_type: ImportType::Static,
                attributes: SmallVec::new(),
            });
            
            lexer.parse_import_attributes(0).unwrap();
            
            // Verify order is preserved
            let import = &lexer.imports[0];
            for (i, (expected_key, expected_value)) in pairs.iter().enumerate() {
                let attr = &import.attributes[i];
                let actual_key = lexer.str_slice(attr.key_start, attr.key_end);
                let actual_value = lexer.str_slice(attr.value_start, attr.value_end);
                
                prop_assert_eq!(actual_key, expected_key.as_str());
                prop_assert_eq!(actual_value, expected_value.as_str());
            }
        }

        #[test]
        fn prop_import_attributes_with_whitespace(
            pairs in prop::collection::vec(arb_attribute_pair(), 1..3),
            ws1 in prop::string::string_regex("[ \t\n]*").unwrap(),
            ws2 in prop::string::string_regex("[ \t\n]*").unwrap(),
            ws3 in prop::string::string_regex("[ \t\n]*").unwrap(),
        ) {
            let pairs_str = pairs
                .iter()
                .map(|(k, v)| format!("{}{}{}:{}{}",  ws1, k, ws2, ws2, v))
                .collect::<Vec<_>>()
                .join(&format!(",{}", ws3));
            let source = format!("with{}{{{}{}{}}}", ws1, ws2, pairs_str, ws3);
            
            let mut lexer = Lexer::new(&source);
            lexer.imports.push(Import {
                start: 0,
                end: 0,
                statement_start: 0,
                statement_end: 0,
                attr_index: None,
                dynamic: None,
                safe: true,
                import_type: ImportType::Static,
                attributes: SmallVec::new(),
            });
            
            let result = lexer.parse_import_attributes(0);
            
            // Should parse successfully despite whitespace
            prop_assert!(result.is_ok());
            prop_assert_eq!(lexer.imports[0].attributes.len(), pairs.len());
        }

        #[test]
        fn prop_import_attributes_string_keys(
            keys in prop::collection::vec(
                prop::string::string_regex("[a-zA-Z0-9_-]+").unwrap(),
                1..4
            ),
            values in prop::collection::vec(
                prop::string::string_regex("[a-zA-Z0-9_./:-]+").unwrap(),
                1..4
            )
        ) {
            // Ensure same length
            let min_len = keys.len().min(values.len());
            let keys = &keys[..min_len];
            let values = &values[..min_len];
            
            let pairs_str = keys
                .iter()
                .zip(values.iter())
                .map(|(k, v)| format!("\"{}\": \"{}\"", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            let source = format!("with {{ {} }}", pairs_str);
            
            let mut lexer = Lexer::new(&source);
            lexer.imports.push(Import {
                start: 0,
                end: 0,
                statement_start: 0,
                statement_end: 0,
                attr_index: None,
                dynamic: None,
                safe: true,
                import_type: ImportType::Static,
                attributes: SmallVec::new(),
            });
            
            let result = lexer.parse_import_attributes(0);
            prop_assert!(result.is_ok());
            prop_assert_eq!(lexer.imports[0].attributes.len(), min_len);
        }

        #[test]
        fn prop_import_attributes_empty_with_clause(
            ws in prop::string::string_regex("[ \t\n]*").unwrap()
        ) {
            let source = format!("with{}{{{}}}", ws, ws);
            
            let mut lexer = Lexer::new(&source);
            lexer.imports.push(Import {
                start: 0,
                end: 0,
                statement_start: 0,
                statement_end: 0,
                attr_index: None,
                dynamic: None,
                safe: true,
                import_type: ImportType::Static,
                attributes: SmallVec::new(),
            });
            
            let result = lexer.parse_import_attributes(0);
            prop_assert!(result.is_ok());
            prop_assert_eq!(lexer.imports[0].attributes.len(), 0);
        }

        #[test]
        fn prop_import_attributes_no_with_keyword(
            pairs in prop::collection::vec(arb_attribute_pair(), 1..3)
        ) {
            let pairs_str = pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            // Missing "with" keyword
            let source = format!("{{ {} }}", pairs_str);
            
            let mut lexer = Lexer::new(&source);
            lexer.imports.push(Import {
                start: 0,
                end: 0,
                statement_start: 0,
                statement_end: 0,
                attr_index: None,
                dynamic: None,
                safe: true,
                import_type: ImportType::Static,
                attributes: SmallVec::new(),
            });
            
            let result = lexer.parse_import_attributes(0);
            
            // Should succeed but not parse any attributes
            prop_assert!(result.is_ok());
            prop_assert_eq!(lexer.imports[0].attributes.len(), 0);
        }

        #[test]
        fn prop_import_attributes_trailing_comma(
            pairs in prop::collection::vec(arb_attribute_pair(), 1..3)
        ) {
            let pairs_str = pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            let source = format!("with {{ {}, }}", pairs_str);
            
            let mut lexer = Lexer::new(&source);
            lexer.imports.push(Import {
                start: 0,
                end: 0,
                statement_start: 0,
                statement_end: 0,
                attr_index: None,
                dynamic: None,
                safe: true,
                import_type: ImportType::Static,
                attributes: SmallVec::new(),
            });
            
            let result = lexer.parse_import_attributes(0);
            
            // Should parse successfully with trailing comma
            prop_assert!(result.is_ok());
            prop_assert_eq!(lexer.imports[0].attributes.len(), pairs.len());
        }
    }
}
