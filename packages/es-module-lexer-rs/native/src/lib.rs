//! Napi bindings for es-module-lexer.

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[cfg(test)]
mod utf16_tests;

/// JavaScript Import specifier structure.
#[napi(object)]
pub struct JsImport {
    /// Module specifier (if safe string)
    pub n: Option<String>,
    /// Import type
    pub t: u8,
    /// Module specifier start position (UTF-16 code units)
    pub s: u32,
    /// Module specifier end position (UTF-16 code units)
    pub e: u32,
    /// Statement start position (UTF-16 code units)
    pub ss: u32,
    /// Statement end position (UTF-16 code units)
    pub se: u32,
    /// Dynamic import position (UTF-16 code units)
    pub d: i32,
    /// Attributes index (UTF-16 code units)
    pub a: i32,
    /// Import attributes as array of [key, value] pairs
    pub at: Either<Vec<Vec<String>>, Null>,
}

/// JavaScript Export specifier structure.
#[napi(object)]
pub struct JsExport {
    /// Export name
    pub n: String,
    /// Local name
    pub ln: Option<String>,
    /// Export name start position (UTF-16 code units)
    pub s: u32,
    /// Export name end position (UTF-16 code units)
    pub e: u32,
    /// Local name start position (UTF-16 code units)
    pub ls: i32,
    /// Local name end position (UTF-16 code units)
    pub le: i32,
}

/// JavaScript parse result structure.
#[napi(object)]
pub struct JsParseResult {
    pub imports: Vec<JsImport>,
    pub exports: Vec<JsExport>,
    pub facade: bool,
    pub has_module_syntax: bool,
}

/// Convert UTF-8 byte index to UTF-16 code unit index using a precomputed map.
/// 
/// JavaScript uses UTF-16 encoding, while Rust uses UTF-8.
/// This function converts byte positions to character positions
/// that JavaScript can use with string.slice().
pub(crate) fn build_utf16_index_map(source: &str) -> Vec<usize> {
    let mut map = Vec::with_capacity(source.len() + 1);
    let mut utf16_index = 0;
    
    map.push(0); // Position 0 maps to 0
    
    for ch in source.chars() {
        let utf8_len = ch.len_utf8();
        let utf16_len = ch.len_utf16();
        
        // Add mapping for each byte in this character
        // All bytes of the same character map to the same UTF-16 position
        for _ in 0..utf8_len {
            map.push(utf16_index + utf16_len);
        }
        
        utf16_index += utf16_len;
    }
    
    map
}

/// Convert a Rust Import to JavaScript JsImport using precomputed UTF-16 index map.
fn convert_import(source: &str, imp: es_module_lexer::Import, utf16_map: &[usize]) -> JsImport {
    // Convert UTF-8 byte indices to UTF-16 code unit indices using the map
    // Clamp indices to valid range
    let s = utf16_map[imp.start.min(utf16_map.len() - 1)] as u32;
    let e = utf16_map[imp.end.min(utf16_map.len() - 1)] as u32;
    let ss = utf16_map[imp.statement_start.min(utf16_map.len() - 1)] as u32;
    let se = utf16_map[imp.statement_end.min(utf16_map.len() - 1)] as u32;
    
    // Extract module specifier if safe
    let n = if imp.safe && imp.start < source.len() && imp.end <= source.len() {
        Some(source[imp.start..imp.end].to_string())
    } else {
        None
    };
    
    // Convert dynamic import position
    let d = imp
        .dynamic
        .map(|pos| utf16_map[pos.min(utf16_map.len() - 1)] as i32)
        .unwrap_or(-1);
    
    // Convert attributes index
    let a = imp
        .attr_index
        .map(|pos| utf16_map[pos.min(utf16_map.len() - 1)] as i32)
        .unwrap_or(-1);
    
    // Convert attributes to JavaScript format: [[key, value], ...]
    // Use Either to ensure the field is always present (as array or null)
    let at = if imp.attributes.is_empty() {
        Either::B(Null)
    } else {
        Either::A(
            imp.attributes
                .into_iter()
                .map(|attr| {
                    vec![
                        source[attr.key_start..attr.key_end].to_string(),
                        source[attr.value_start..attr.value_end].to_string(),
                    ]
                })
                .collect(),
        )
    };

    JsImport {
        n,
        t: imp.import_type as u8,
        s,
        e,
        ss,
        se,
        d,
        a,
        at,
    }
}

/// Convert a Rust Export to JavaScript JsExport using precomputed UTF-16 index map.
fn convert_export(source: &str, exp: es_module_lexer::Export, utf16_map: &[usize]) -> JsExport {
    // Convert UTF-8 byte indices to UTF-16 code unit indices using the map
    // Clamp indices to valid range
    let s = utf16_map[exp.start.min(utf16_map.len() - 1)] as u32;
    let e = utf16_map[exp.end.min(utf16_map.len() - 1)] as u32;
    
    // Extract export name
    let n = if exp.start < source.len() && exp.end <= source.len() {
        source[exp.start..exp.end].to_string()
    } else {
        String::new()
    };
    
    // Extract local name if different
    let ln = exp
        .local_start
        .zip(exp.local_end)
        .filter(|(start, end)| *start < source.len() && *end <= source.len())
        .map(|(start, end)| source[start..end].to_string());
    
    // Convert local name positions
    let ls = exp
        .local_start
        .map(|pos| utf16_map[pos.min(utf16_map.len() - 1)] as i32)
        .unwrap_or(-1);
    let le = exp
        .local_end
        .map(|pos| utf16_map[pos.min(utf16_map.len() - 1)] as i32)
        .unwrap_or(-1);
    
    JsExport { n, ln, s, e, ls, le }
}

/// Parse JavaScript source code to extract imports and exports.
/// 
/// # Arguments
/// 
/// * `source` - The JavaScript source code to parse
/// * `_name` - Optional file name (for error messages, currently unused)
/// 
/// # Returns
/// 
/// A `JsParseResult` containing:
/// - `imports`: Array of import specifiers
/// - `exports`: Array of export specifiers
/// - `facade`: Whether this is a facade module (pure imports/exports)
/// - `has_module_syntax`: Whether the file contains any module syntax
/// 
/// # Errors
/// 
/// Returns an error if the source code contains syntax errors.
#[napi]
pub fn parse(source: String, _name: Option<String>) -> Result<JsParseResult> {
    // Parse the source code
    let result = es_module_lexer::parse(&source)
        .map_err(|e| Error::new(Status::GenericFailure, format!("{}", e)))?;

    // Build UTF-16 index map once for all conversions
    let utf16_map = build_utf16_index_map(&source);

    // Convert imports with UTF-16 index conversion
    let imports = result
        .imports
        .into_iter()
        .map(|imp| convert_import(&source, imp, &utf16_map))
        .collect();

    // Convert exports with UTF-16 index conversion
    let exports = result
        .exports
        .into_iter()
        .map(|exp| convert_export(&source, exp, &utf16_map))
        .collect();

    Ok(JsParseResult {
        imports,
        exports,
        facade: result.facade,
        has_module_syntax: result.has_module_syntax,
    })
}
