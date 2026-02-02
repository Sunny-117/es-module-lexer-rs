//! Core data structures for the lexer.

use smallvec::SmallVec;

/// Represents an import statement in the source code.
#[derive(Debug, Clone)]
pub struct Import {
    /// Module specifier start position (byte index)
    pub start: usize,
    /// Module specifier end position (byte index)
    pub end: usize,
    /// Import statement start position
    pub statement_start: usize,
    /// Import statement end position
    pub statement_end: usize,
    /// Import attributes start position (if present)
    pub attr_index: Option<usize>,
    /// Dynamic import marker: None=static, Some(pos)=dynamic
    pub dynamic: Option<usize>,
    /// Whether this is a safe string literal
    pub safe: bool,
    /// Import type
    pub import_type: ImportType,
    /// Import attributes (using SmallVec to avoid heap allocation for common case of 0-2 attributes)
    pub attributes: SmallVec<[Attribute; 2]>,
}

/// Type of import statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImportType {
    Static = 1,
    Dynamic = 2,
    ImportMeta = 3,
    StaticSourcePhase = 4,
    DynamicSourcePhase = 5,
    StaticDeferPhase = 6,
    DynamicDeferPhase = 7,
}

/// Represents an export statement in the source code.
#[derive(Debug, Clone)]
pub struct Export {
    /// Export name start position
    pub start: usize,
    /// Export name end position
    pub end: usize,
    /// Local name start position (if different from export name)
    pub local_start: Option<usize>,
    /// Local name end position
    pub local_end: Option<usize>,
}

/// Represents an import attribute (with clause).
#[derive(Debug, Clone)]
pub struct Attribute {
    /// Attribute key start position
    pub key_start: usize,
    /// Attribute key end position
    pub key_end: usize,
    /// Attribute value start position
    pub value_start: usize,
    /// Attribute value end position
    pub value_end: usize,
}

/// State of an open token (parenthesis, brace, template).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenTokenState {
    AnyParen,
    AnyBrace,
    Template,
    TemplateBrace,
    ImportParen,
    ClassBrace,
    AsyncParen,
}

/// Represents an open token on the stack.
#[derive(Debug, Clone, Copy)]
pub struct OpenToken {
    pub state: OpenTokenState,
    pub pos: usize,
}

/// Result of parsing a JavaScript module.
#[derive(Debug)]
pub struct ParseResult {
    /// All import statements
    pub imports: Vec<Import>,
    /// All export statements
    pub exports: Vec<Export>,
    /// Whether this is a facade module (pure module file)
    pub facade: bool,
    /// Whether the file contains module syntax
    pub has_module_syntax: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_creation() {
        let import = Import {
            start: 10,
            end: 20,
            statement_start: 0,
            statement_end: 30,
            attr_index: Some(25),
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        };

        assert_eq!(import.start, 10);
        assert_eq!(import.end, 20);
        assert_eq!(import.statement_start, 0);
        assert_eq!(import.statement_end, 30);
        assert_eq!(import.attr_index, Some(25));
        assert_eq!(import.dynamic, None);
        assert!(import.safe);
        assert_eq!(import.import_type, ImportType::Static);
        assert_eq!(import.attributes.len(), 0);
    }

    #[test]
    fn test_import_type_variants() {
        assert_eq!(ImportType::Static as u8, 1);
        assert_eq!(ImportType::Dynamic as u8, 2);
        assert_eq!(ImportType::ImportMeta as u8, 3);
        assert_eq!(ImportType::StaticSourcePhase as u8, 4);
        assert_eq!(ImportType::DynamicSourcePhase as u8, 5);
        assert_eq!(ImportType::StaticDeferPhase as u8, 6);
        assert_eq!(ImportType::DynamicDeferPhase as u8, 7);
    }

    #[test]
    fn test_import_type_equality() {
        let type1 = ImportType::Static;
        let type2 = ImportType::Static;
        let type3 = ImportType::Dynamic;

        assert_eq!(type1, type2);
        assert_ne!(type1, type3);
    }

    #[test]
    fn test_export_creation() {
        let export = Export {
            start: 5,
            end: 15,
            local_start: Some(20),
            local_end: Some(30),
        };

        assert_eq!(export.start, 5);
        assert_eq!(export.end, 15);
        assert_eq!(export.local_start, Some(20));
        assert_eq!(export.local_end, Some(30));
    }

    #[test]
    fn test_export_without_local_name() {
        let export = Export {
            start: 5,
            end: 15,
            local_start: None,
            local_end: None,
        };

        assert_eq!(export.local_start, None);
        assert_eq!(export.local_end, None);
    }

    #[test]
    fn test_attribute_creation() {
        let attr = Attribute {
            key_start: 0,
            key_end: 4,
            value_start: 6,
            value_end: 12,
        };

        assert_eq!(attr.key_start, 0);
        assert_eq!(attr.key_end, 4);
        assert_eq!(attr.value_start, 6);
        assert_eq!(attr.value_end, 12);
    }

    #[test]
    fn test_open_token_state_variants() {
        let states = [
            OpenTokenState::AnyParen,
            OpenTokenState::AnyBrace,
            OpenTokenState::Template,
            OpenTokenState::TemplateBrace,
            OpenTokenState::ImportParen,
            OpenTokenState::ClassBrace,
            OpenTokenState::AsyncParen,
        ];

        // Test that all variants can be created
        for state in &states {
            let token = OpenToken {
                state: *state,
                pos: 0,
            };
            assert_eq!(token.pos, 0);
        }
    }

    #[test]
    fn test_open_token_state_equality() {
        let state1 = OpenTokenState::AnyParen;
        let state2 = OpenTokenState::AnyParen;
        let state3 = OpenTokenState::AnyBrace;

        assert_eq!(state1, state2);
        assert_ne!(state1, state3);
    }

    #[test]
    fn test_open_token_creation() {
        let token = OpenToken {
            state: OpenTokenState::ImportParen,
            pos: 42,
        };

        assert_eq!(token.pos, 42);
        assert_eq!(token.state, OpenTokenState::ImportParen);
    }

    #[test]
    fn test_parse_result_creation() {
        let import = Import {
            start: 10,
            end: 20,
            statement_start: 0,
            statement_end: 30,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        };

        let export = Export {
            start: 5,
            end: 15,
            local_start: None,
            local_end: None,
        };

        let result = ParseResult {
            imports: vec![import],
            exports: vec![export],
            facade: true,
            has_module_syntax: true,
        };

        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.exports.len(), 1);
        assert!(result.facade);
        assert!(result.has_module_syntax);
    }

    #[test]
    fn test_import_with_attributes() {
        let attr1 = Attribute {
            key_start: 0,
            key_end: 4,
            value_start: 6,
            value_end: 12,
        };

        let attr2 = Attribute {
            key_start: 14,
            key_end: 18,
            value_start: 20,
            value_end: 26,
        };

        let import = Import {
            start: 10,
            end: 20,
            statement_start: 0,
            statement_end: 30,
            attr_index: Some(25),
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::from_vec(vec![attr1, attr2]),
        };

        assert_eq!(import.attributes.len(), 2);
        assert_eq!(import.attributes[0].key_start, 0);
        assert_eq!(import.attributes[1].key_start, 14);
    }

    #[test]
    fn test_dynamic_import() {
        let import = Import {
            start: 10,
            end: 20,
            statement_start: 0,
            statement_end: 30,
            attr_index: None,
            dynamic: Some(5),
            safe: false,
            import_type: ImportType::Dynamic,
            attributes: SmallVec::new(),
        };

        assert_eq!(import.dynamic, Some(5));
        assert!(!import.safe);
        assert_eq!(import.import_type, ImportType::Dynamic);
    }

    #[test]
    fn test_clone_import() {
        let import = Import {
            start: 10,
            end: 20,
            statement_start: 0,
            statement_end: 30,
            attr_index: None,
            dynamic: None,
            safe: true,
            import_type: ImportType::Static,
            attributes: SmallVec::new(),
        };

        let cloned = import.clone();
        assert_eq!(cloned.start, import.start);
        assert_eq!(cloned.import_type, import.import_type);
    }

    #[test]
    fn test_clone_export() {
        let export = Export {
            start: 5,
            end: 15,
            local_start: Some(20),
            local_end: Some(30),
        };

        let cloned = export.clone();
        assert_eq!(cloned.start, export.start);
        assert_eq!(cloned.local_start, export.local_start);
    }
}
