import { describe, it, expect } from 'vitest';
import { parse, ImportType, type ImportSpecifier, type ExportSpecifier } from '../src/index';

describe('TypeScript Wrapper API', () => {
  it('should export parse function', () => {
    expect(parse).toBeDefined();
    expect(typeof parse).toBe('function');
  });

  it('should export ImportType enum', () => {
    expect(ImportType).toBeDefined();
    expect(ImportType.Static).toBe(1);
    expect(ImportType.Dynamic).toBe(2);
    expect(ImportType.ImportMeta).toBe(3);
    expect(ImportType.StaticSourcePhase).toBe(4);
    expect(ImportType.DynamicSourcePhase).toBe(5);
    expect(ImportType.StaticDeferPhase).toBe(6);
    expect(ImportType.DynamicDeferPhase).toBe(7);
  });

  it('should return tuple format compatible with es-module-lexer', () => {
    const source = `import foo from 'bar';`;
    const result = parse(source);
    
    // Should be a tuple
    expect(Array.isArray(result)).toBe(true);
    expect(result).toHaveLength(4);
    
    // Destructure like es-module-lexer
    const [imports, exports, facade, hasModuleSyntax] = result;
    
    expect(Array.isArray(imports)).toBe(true);
    expect(Array.isArray(exports)).toBe(true);
    expect(typeof facade).toBe('boolean');
    expect(typeof hasModuleSyntax).toBe('boolean');
  });

  it('should parse static import', () => {
    const source = `import foo from 'bar';`;
    const [imports, exports, facade, hasModuleSyntax] = parse(source);
    
    expect(imports).toHaveLength(1);
    expect(imports[0].n).toBe('bar');
    expect(imports[0].t).toBe(ImportType.Static);
    expect(imports[0].d).toBe(-1);
    expect(hasModuleSyntax).toBe(true);
  });

  it('should parse dynamic import', () => {
    const source = `import('module');`;
    const [imports] = parse(source);
    
    expect(imports.length).toBeGreaterThan(0);
    const dynamicImport = imports.find(imp => imp.t === ImportType.Dynamic);
    expect(dynamicImport).toBeDefined();
    expect(dynamicImport!.n).toBe('module');
    expect(dynamicImport!.d).toBeGreaterThanOrEqual(0);
  });

  it('should parse export', () => {
    const source = `export const x = 1;`;
    const [imports, exports, facade, hasModuleSyntax] = parse(source);
    
    expect(exports.length).toBeGreaterThan(0);
    expect(exports[0].n).toBe('x');
    expect(hasModuleSyntax).toBe(true);
  });

  it('should handle empty source', () => {
    const [imports, exports, facade, hasModuleSyntax] = parse('');
    
    expect(imports).toEqual([]);
    expect(exports).toEqual([]);
    expect(facade).toBe(true);
    expect(hasModuleSyntax).toBe(false);
  });

  it('should accept optional name parameter', () => {
    const result1 = parse('', 'test.js');
    expect(result1).toBeDefined();
    
    const result2 = parse('');
    expect(result2).toBeDefined();
  });

  it('should have correct TypeScript types', () => {
    const source = `import foo from 'bar'; export const x = 1;`;
    const [imports, exports] = parse(source);
    
    // Type assertions to verify TypeScript types
    const imp: ImportSpecifier = imports[0];
    expect(imp.n).toBeDefined();
    expect(typeof imp.t).toBe('number');
    expect(typeof imp.s).toBe('number');
    expect(typeof imp.e).toBe('number');
    expect(typeof imp.ss).toBe('number');
    expect(typeof imp.se).toBe('number');
    expect(typeof imp.d).toBe('number');
    expect(typeof imp.a).toBe('number');
    
    const exp: ExportSpecifier = exports[0];
    expect(typeof exp.n).toBe('string');
    expect(typeof exp.s).toBe('number');
    expect(typeof exp.e).toBe('number');
    expect(typeof exp.ls).toBe('number');
    expect(typeof exp.le).toBe('number');
  });

  it('should handle UTF-16 positions correctly', () => {
    // Test with ASCII
    const source1 = `import 'bar';`;
    const [imports1] = parse(source1);
    expect(imports1[0].s).toBe(8);
    expect(imports1[0].e).toBe(11);
    
    // Test with emoji (4 bytes UTF-8, 2 UTF-16 code units)
    const source2 = `import '😀';`;
    const [imports2] = parse(source2);
    expect(imports2[0].s).toBe(8);
    expect(imports2[0].e).toBe(10); // 8 + 2 UTF-16 code units
  });

  it('should be compatible with es-module-lexer API', () => {
    // This test verifies that our API matches es-module-lexer's API
    const source = `
      import foo from 'bar';
      import { x, y } from 'baz';
      export const z = 1;
      export default function() {}
    `;
    
    const [imports, exports, facade, hasModuleSyntax] = parse(source);
    
    // Verify structure matches es-module-lexer
    expect(imports.length).toBeGreaterThan(0);
    expect(exports.length).toBeGreaterThan(0);
    expect(typeof facade).toBe('boolean');
    expect(hasModuleSyntax).toBe(true);
    
    // Verify import structure
    imports.forEach(imp => {
      expect(imp).toHaveProperty('n');
      expect(imp).toHaveProperty('t');
      expect(imp).toHaveProperty('s');
      expect(imp).toHaveProperty('e');
      expect(imp).toHaveProperty('ss');
      expect(imp).toHaveProperty('se');
      expect(imp).toHaveProperty('d');
      expect(imp).toHaveProperty('a');
    });
    
    // Verify export structure
    exports.forEach(exp => {
      expect(exp).toHaveProperty('n');
      expect(exp).toHaveProperty('s');
      expect(exp).toHaveProperty('e');
      expect(exp).toHaveProperty('ls');
      expect(exp).toHaveProperty('le');
    });
  });
});
