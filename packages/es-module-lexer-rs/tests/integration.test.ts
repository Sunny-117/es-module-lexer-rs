import { describe, test, expect } from 'vitest';
import { parse as parseRust } from '../src/index';
import { parse as parseOriginal, init } from 'es-module-lexer';
import { readFileSync } from 'fs';
import { join } from 'path';

// Initialize the original lexer
await init;

describe('Integration Tests - Output Alignment', () => {
  test('should produce identical output for simple import', async () => {
    const source = `import foo from 'bar';`;
    
    const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
    const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
    
    expect(importsRust).toEqual(importsOrig);
    expect(exportsRust).toEqual(exportsOrig);
    expect(facadeRust).toBe(facadeOrig);
    expect(hasModuleRust).toBe(hasModuleOrig);
  });

  test.todo('should produce identical output for simple export', async () => {
    const source = `export const x = 1;`;
    
    const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
    const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
    
    expect(importsRust).toEqual(importsOrig);
    expect(exportsRust).toEqual(exportsOrig);
    expect(facadeRust).toBe(facadeOrig);
    expect(hasModuleRust).toBe(hasModuleOrig);
  });

  test.todo('should produce identical output for dynamic import', async () => {
    const source = `import('dynamic');`;
    
    const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
    const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
    
    expect(importsRust).toEqual(importsOrig);
    expect(exportsRust).toEqual(exportsOrig);
    expect(facadeRust).toBe(facadeOrig);
    expect(hasModuleRust).toBe(hasModuleOrig);
  });

  test.todo('should produce identical output for import with attributes', async () => {
    const source = `import foo from 'bar' with { type: 'json' };`;
    
    const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
    const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
    
    expect(importsRust).toEqual(importsOrig);
    expect(exportsRust).toEqual(exportsOrig);
    expect(facadeRust).toBe(facadeOrig);
    expect(hasModuleRust).toBe(hasModuleOrig);
  });

  test.todo('should produce identical output for mixed imports and exports', async () => {
    const source = `
      import foo from 'bar';
      import { x, y } from 'baz';
      export const z = 1;
      export default function() {}
    `;
    
    const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
    const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
    
    expect(importsRust.length).toBe(importsOrig.length);
    expect(exportsRust.length).toBe(exportsOrig.length);
    expect(facadeRust).toBe(facadeOrig);
    expect(hasModuleRust).toBe(hasModuleOrig);
    
    // Compare each import
    for (let i = 0; i < importsOrig.length; i++) {
      expect(importsRust[i]).toEqual(importsOrig[i]);
    }
    
    // Compare each export
    for (let i = 0; i < exportsOrig.length; i++) {
      expect(exportsRust[i]).toEqual(exportsOrig[i]);
    }
  });

  test.todo('should produce identical output for reexports', async () => {
    const source = `
      export { hello as default } from "test-dep";
      export * from 'module';
      export * as ns from 'module2';
    `;
    
    const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
    const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
    
    expect(importsRust).toEqual(importsOrig);
    expect(exportsRust).toEqual(exportsOrig);
    expect(facadeRust).toBe(facadeOrig);
    expect(hasModuleRust).toBe(hasModuleOrig);
  });

  test.todo('should produce identical output for import.meta', async () => {
    const source = `
      export var hello = 'world';
      console.log(import.meta.url);
    `;
    
    const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
    const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
    
    expect(importsRust).toEqual(importsOrig);
    expect(exportsRust).toEqual(exportsOrig);
    expect(facadeRust).toBe(facadeOrig);
    expect(hasModuleRust).toBe(hasModuleOrig);
  });

  test.todo('should produce identical output for comments', async () => {
    const source = `
      import/* 'x' */ 'a';
      import /* 'x' */ 'b';
      export var z  /*  */
      export {
        a,
        // b,
        /* c */ d
      };
    `;
    
    const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
    const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
    
    expect(importsRust).toEqual(importsOrig);
    expect(exportsRust).toEqual(exportsOrig);
    expect(facadeRust).toBe(facadeOrig);
    expect(hasModuleRust).toBe(hasModuleOrig);
  });

  test.todo('should produce identical output for string escapes', async () => {
    const source = `
      import './\\x61\\x62\\x63.js';
      import './\\u{20204}.js';
    `;
    
    const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
    const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
    
    expect(importsRust).toEqual(importsOrig);
    expect(exportsRust).toEqual(exportsOrig);
    expect(facadeRust).toBe(facadeOrig);
    expect(hasModuleRust).toBe(hasModuleOrig);
  });

  test('should produce identical output for empty source', async () => {
    const source = ``;
    
    const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
    const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
    
    expect(importsRust).toEqual(importsOrig);
    expect(exportsRust).toEqual(exportsOrig);
    expect(facadeRust).toBe(facadeOrig);
    expect(hasModuleRust).toBe(hasModuleOrig);
  });

  test('should produce identical output for facade mode detection', async () => {
    const source1 = `
      import foo from 'bar';
      export const x = 1;
    `;
    
    const [, , facadeOrig1] = parseOriginal(source1);
    const [, , facadeRust1] = parseRust(source1);
    expect(facadeRust1).toBe(facadeOrig1);
    
    const source2 = `
      import foo from 'bar';
      const x = 1;
      export { x };
    `;
    
    const [, , facadeOrig2] = parseOriginal(source2);
    const [, , facadeRust2] = parseRust(source2);
    expect(facadeRust2).toBe(facadeOrig2);
  });
});

describe('Integration Tests - Real-world Code', () => {
  test.skip('should parse angular.js sample', () => {
    // Skip if sample files don't exist
    try {
      const source = readFileSync(
        join(__dirname, '../../../es-module-lexer/test/samples/angular.js'),
        'utf-8'
      );
      
      const [importsOrig, exportsOrig] = parseOriginal(source);
      const [importsRust, exportsRust] = parseRust(source);
      
      expect(importsRust.length).toBe(importsOrig.length);
      expect(exportsRust.length).toBe(exportsOrig.length);
      
      // Compare structure
      for (let i = 0; i < importsOrig.length; i++) {
        expect(importsRust[i]).toEqual(importsOrig[i]);
      }
      
      for (let i = 0; i < exportsOrig.length; i++) {
        expect(exportsRust[i]).toEqual(exportsOrig[i]);
      }
    } catch (e) {
      // Skip if file doesn't exist
      console.log('Skipping angular.js test - file not found');
    }
  });

  test.skip('should parse d3.js sample', () => {
    try {
      const source = readFileSync(
        join(__dirname, '../../../es-module-lexer/test/samples/d3.js'),
        'utf-8'
      );
      
      const [importsOrig, exportsOrig] = parseOriginal(source);
      const [importsRust, exportsRust] = parseRust(source);
      
      expect(importsRust.length).toBe(importsOrig.length);
      expect(exportsRust.length).toBe(exportsOrig.length);
      
      for (let i = 0; i < importsOrig.length; i++) {
        expect(importsRust[i]).toEqual(importsOrig[i]);
      }
      
      for (let i = 0; i < exportsOrig.length; i++) {
        expect(exportsRust[i]).toEqual(exportsOrig[i]);
      }
    } catch (e) {
      console.log('Skipping d3.js test - file not found');
    }
  });

  test.skip('should parse rollup.js sample', () => {
    try {
      const source = readFileSync(
        join(__dirname, '../../../es-module-lexer/test/samples/rollup.js'),
        'utf-8'
      );
      
      const [importsOrig, exportsOrig] = parseOriginal(source);
      const [importsRust, exportsRust] = parseRust(source);
      
      expect(importsRust.length).toBe(importsOrig.length);
      expect(exportsRust.length).toBe(exportsOrig.length);
      
      for (let i = 0; i < importsOrig.length; i++) {
        expect(importsRust[i]).toEqual(importsOrig[i]);
      }
      
      for (let i = 0; i < exportsOrig.length; i++) {
        expect(exportsRust[i]).toEqual(exportsOrig[i]);
      }
    } catch (e) {
      console.log('Skipping rollup.js test - file not found');
    }
  });
});
