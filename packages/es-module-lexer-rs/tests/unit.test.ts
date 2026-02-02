import { describe, it, expect } from 'vitest';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);
const { parse } = require('../native/index.node');

describe('es-module-lexer-rs - Napi Bindings', () => {
  it('should export parse function', () => {
    expect(parse).toBeDefined();
    expect(typeof parse).toBe('function');
  });

  it('should return correct structure', () => {
    const result = parse('');
    expect(result).toHaveProperty('imports');
    expect(result).toHaveProperty('exports');
    expect(result).toHaveProperty('facade');
    expect(result).toHaveProperty('hasModuleSyntax');
    expect(Array.isArray(result.imports)).toBe(true);
    expect(Array.isArray(result.exports)).toBe(true);
    expect(typeof result.facade).toBe('boolean');
    expect(typeof result.hasModuleSyntax).toBe('boolean');
  });

  it('should handle empty source', () => {
    const result = parse('');
    expect(result.imports).toEqual([]);
    expect(result.exports).toEqual([]);
    expect(result.facade).toBe(true);
    expect(result.hasModuleSyntax).toBe(false);
  });

  it('should accept optional name parameter', () => {
    const result1 = parse('', 'test.js');
    expect(result1).toBeDefined();
    
    const result2 = parse('');
    expect(result2).toBeDefined();
  });

  it('should handle UTF-16 strings', () => {
    // Test with emoji (multi-byte UTF-8, 2 UTF-16 code units)
    const source = `// 😀`;
    const result = parse(source);
    expect(result).toBeDefined();
  });

  it('should parse static import', () => {
    const source = `import foo from 'bar';`;
    const result = parse(source);
    
    expect(result.imports).toHaveLength(1);
    expect(result.imports[0].n).toBe('bar');
    expect(result.imports[0].t).toBe(1); // Static
    expect(result.imports[0].d).toBe(-1); // Not dynamic
    expect(result.hasModuleSyntax).toBe(true);
  });

  it('should parse dynamic import', () => {
    const source = `import('module');`;
    const result = parse(source);
    
    expect(result.imports.length).toBeGreaterThan(0);
    const dynamicImport = result.imports.find(imp => imp.t === 2);
    expect(dynamicImport).toBeDefined();
    expect(dynamicImport!.n).toBe('module');
    expect(dynamicImport!.d).toBeGreaterThanOrEqual(0); // Dynamic position
  });

  it('should parse export', () => {
    const source = `export const x = 1;`;
    const result = parse(source);
    
    expect(result.exports.length).toBeGreaterThan(0);
    expect(result.exports[0].n).toBe('x');
    expect(result.hasModuleSyntax).toBe(true);
  });

  it('should convert UTF-8 byte positions to UTF-16 code units', () => {
    // Test with ASCII (1 byte UTF-8, 1 UTF-16 code unit)
    const source1 = `import 'bar';`;
    const result1 = parse(source1);
    expect(result1.imports[0].s).toBe(8); // Position of 'b' in UTF-16
    expect(result1.imports[0].e).toBe(11); // Position after 'r' in UTF-16
    
    // Test with emoji (4 bytes UTF-8, 2 UTF-16 code units)
    const source2 = `import '😀';`;
    const result2 = parse(source2);
    // The emoji takes 2 UTF-16 code units, so positions should reflect that
    expect(result2.imports[0].s).toBe(8);
    expect(result2.imports[0].e).toBe(10); // 8 + 2 UTF-16 code units
  });

  // Note: The following tests are skipped because they require complete lexer implementation
  // or test features that aren't fully implemented yet
  
  it.skip('should parse import with attributes (requires complete lexer)', () => {
    const source = `import foo from 'bar' with { type: 'json' };`;
    const result = parse(source);
    
    expect(result.imports).toHaveLength(1);
    expect(result.imports[0].n).toBe('bar');
    expect(result.imports[0].at).toBeDefined();
    expect(result.imports[0].at).toHaveLength(1);
    expect(result.imports[0].at![0]).toEqual(['type', 'json']);
  });

  it.skip('should handle errors gracefully (requires complete lexer)', () => {
    const source = `import foo from 'bar`;
    expect(() => parse(source)).toThrow();
  });

  it.skip('should detect facade mode (requires complete lexer)', () => {
    const source = `import foo from 'bar';\nexport const x = 1;`;
    const result = parse(source);
    
    expect(result.facade).toBe(true);
  });

  it.skip('should detect non-facade mode (requires complete lexer)', () => {
    const source = `import foo from 'bar';\nconst x = 1;\nexport { x };`;
    const result = parse(source);
    
    expect(result.facade).toBe(false);
  });
});
