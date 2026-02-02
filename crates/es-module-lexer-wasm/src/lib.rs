//! WebAssembly bindings for es-module-lexer.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// JavaScript Import specifier structure.
#[derive(Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct JsImport {
    /// Module specifier (if safe string)
    #[wasm_bindgen(skip)]
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
}

#[wasm_bindgen]
impl JsImport {
    #[wasm_bindgen(getter)]
    pub fn n(&self) -> JsValue {
        match &self.n {
            Some(s) => JsValue::from_str(s),
            None => JsValue::UNDEFINED,
        }
    }
}

/// JavaScript Export specifier structure.
#[derive(Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct JsExport {
    /// Export name
    pub n: String,
    /// Export name start position (UTF-16 code units)
    pub s: u32,
    /// Export name end position (UTF-16 code units)
    pub e: u32,
    /// Local name start position (UTF-16 code units)
    pub ls: i32,
    /// Local name end position (UTF-16 code units)
    pub le: i32,
}

#[wasm_bindgen]
impl JsExport {
    #[wasm_bindgen(getter)]
    pub fn ln(&self) -> JsValue {
        JsValue::UNDEFINED
    }
}

/// Build UTF-16 index map for efficient position conversion.
fn build_utf16_index_map(source: &str) -> Vec<usize> {
    let mut map = Vec::with_capacity(source.len() + 1);
    let mut utf16_index = 0;
    
    map.push(0);
    
    for ch in source.chars() {
        let utf8_len = ch.len_utf8();
        let utf16_len = ch.len_utf16();
        
        for _ in 0..utf8_len {
            map.push(utf16_index + utf16_len);
        }
        
        utf16_index += utf16_len;
    }
    
    map
}

/// Parse JavaScript source code to extract imports and exports.
#[wasm_bindgen]
pub fn parse(source: &str) -> Result<JsValue, JsValue> {
    // Parse the source code
    let result = es_module_lexer::parse(source)
        .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

    // Build UTF-16 index map once
    let utf16_map = build_utf16_index_map(source);

    // Convert imports
    let imports: Vec<JsImport> = result
        .imports
        .into_iter()
        .map(|imp| {
            let s = utf16_map[imp.start] as u32;
            let e = utf16_map[imp.end] as u32;
            let ss = utf16_map[imp.statement_start] as u32;
            let se = utf16_map[imp.statement_end] as u32;
            
            let n = if imp.safe {
                Some(source[imp.start..imp.end].to_string())
            } else {
                None
            };
            
            let d = imp
                .dynamic
                .map(|pos| utf16_map[pos] as i32)
                .unwrap_or(-1);
            
            let a = imp
                .attr_index
                .map(|pos| utf16_map[pos] as i32)
                .unwrap_or(-1);
            
            JsImport {
                n,
                t: imp.import_type as u8,
                s,
                e,
                ss,
                se,
                d,
                a,
            }
        })
        .collect();

    // Convert exports
    let exports: Vec<JsExport> = result
        .exports
        .into_iter()
        .map(|exp| {
            let s = utf16_map[exp.start] as u32;
            let e = utf16_map[exp.end] as u32;
            
            let n = source[exp.start..exp.end].to_string();
            
            let ls = exp
                .local_start
                .map(|pos| utf16_map[pos] as i32)
                .unwrap_or(-1);
            let le = exp
                .local_end
                .map(|pos| utf16_map[pos] as i32)
                .unwrap_or(-1);
            
            JsExport { n, s, e, ls, le }
        })
        .collect();

    // Create result object
    let result_obj = js_sys::Object::new();
    
    // Set imports array
    let imports_array = js_sys::Array::new();
    for import in imports {
        let import_obj = js_sys::Object::new();
        
        // Set properties
        js_sys::Reflect::set(&import_obj, &"t".into(), &import.t.into())?;
        js_sys::Reflect::set(&import_obj, &"s".into(), &import.s.into())?;
        js_sys::Reflect::set(&import_obj, &"e".into(), &import.e.into())?;
        js_sys::Reflect::set(&import_obj, &"ss".into(), &import.ss.into())?;
        js_sys::Reflect::set(&import_obj, &"se".into(), &import.se.into())?;
        js_sys::Reflect::set(&import_obj, &"d".into(), &import.d.into())?;
        js_sys::Reflect::set(&import_obj, &"a".into(), &import.a.into())?;
        
        if let Some(n) = import.n {
            js_sys::Reflect::set(&import_obj, &"n".into(), &n.into())?;
        }
        
        imports_array.push(&import_obj);
    }
    js_sys::Reflect::set(&result_obj, &"imports".into(), &imports_array)?;
    
    // Set exports array
    let exports_array = js_sys::Array::new();
    for export in exports {
        let export_obj = js_sys::Object::new();
        
        js_sys::Reflect::set(&export_obj, &"n".into(), &export.n.into())?;
        js_sys::Reflect::set(&export_obj, &"s".into(), &export.s.into())?;
        js_sys::Reflect::set(&export_obj, &"e".into(), &export.e.into())?;
        js_sys::Reflect::set(&export_obj, &"ls".into(), &export.ls.into())?;
        js_sys::Reflect::set(&export_obj, &"le".into(), &export.le.into())?;
        
        exports_array.push(&export_obj);
    }
    js_sys::Reflect::set(&result_obj, &"exports".into(), &exports_array)?;
    
    // Set facade and has_module_syntax
    js_sys::Reflect::set(&result_obj, &"facade".into(), &result.facade.into())?;
    js_sys::Reflect::set(&result_obj, &"hasModuleSyntax".into(), &result.has_module_syntax.into())?;
    
    Ok(result_obj.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_simple_import() {
        let source = "import foo from 'bar';";
        let result = parse(source);
        assert!(result.is_ok());
    }
}
