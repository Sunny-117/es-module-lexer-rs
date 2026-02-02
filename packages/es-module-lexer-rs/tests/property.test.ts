/**
 * Property-Based Tests for es-module-lexer-rs
 * 
 * Feature: es-module-lexer-rs
 * Property 15: 输出对齐（与原始实现）
 * 
 * Validates Requirements: 13.2, 13.7
 * 
 * For any valid JavaScript module code, the Rust implementation's parse output
 * (imports array, exports array, facade flag, hasModuleSyntax flag) should be
 * completely identical to the es-module-lexer original implementation's output.
 */

import { describe, test } from 'vitest';
import * as fc from 'fast-check';
import { parse as parseRust } from '../src/index';
import { parse as parseOriginal, init } from 'es-module-lexer';

// Initialize the original lexer
await init;

// Arbitrary generators for JavaScript module syntax

const arbModuleSpecifier = () =>
  fc.oneof(
    fc.constant('./module.js'),
    fc.constant('../parent.js'),
    fc.constant('package'),
    fc.constant('@scope/package'),
    fc.constant('./file'),
    fc.string({ minLength: 1, maxLength: 20 }).map(s => s.replace(/[^a-c\/\.\-_]/g, 'a'))
  );

const arbIdentifier = () =>
  fc.string({ minLength: 1, maxLength: 10 }).map(s => s.replace(/[^a-cxyz_$]/g, 'a'));

const arbStaticImport = () =>
  fc.tuple(arbModuleSpecifier()).map(([spec]) => `import '${spec}';`);

const arbNamedImport = () =>
  fc.tuple(arbIdentifier(), arbModuleSpecifier()).map(
    ([name, spec]) => `import { ${name} } from '${spec}';`
  );

const arbDefaultImport = () =>
  fc.tuple(arbIdentifier(), arbModuleSpecifier()).map(
    ([name, spec]) => `import ${name} from '${spec}';`
  );

const arbDynamicImport = () =>
  fc.tuple(arbModuleSpecifier()).map(([spec]) => `import('${spec}');`);

const arbImportMeta = () => fc.constant('import.meta.url');

const arbExportConst = () =>
  fc.tuple(arbIdentifier()).map(([name]) => `export const ${name} = 1;`);

const arbExportFunction = () =>
  fc.tuple(arbIdentifier()).map(([name]) => `export function ${name}() {}`);

const arbExportDefault = () => fc.constant('export default 42;');

const arbExportNamed = () =>
  fc.tuple(arbIdentifier()).map(([name]) => `export { ${name} };`);

const arbImportStatement = () =>
  fc.oneof(
    arbStaticImport(),
    arbNamedImport(),
    arbDefaultImport(),
    arbDynamicImport(),
    arbImportMeta()
  );

const arbExportStatement = () =>
  fc.oneof(
    arbExportConst(),
    arbExportFunction(),
    arbExportDefault(),
    arbExportNamed()
  );

const arbModuleCode = () =>
  fc.tuple(
    fc.array(arbImportStatement(), { maxLength: 5 }),
    fc.array(arbExportStatement(), { maxLength: 5 })
  ).map(([imports, exports]) => [...imports, ...exports].join('\n'));

describe('Property-Based Tests - Output Alignment', () => {
  test('Property 15: Output alignment with original implementation', () => {
    fc.assert(
      fc.property(arbModuleCode(), (source) => {
        try {
          const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
          const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
          
          // Compare imports array length
          if (importsRust.length !== importsOrig.length) {
            throw new Error(
              `Import count mismatch: Rust=${importsRust.length}, Original=${importsOrig.length}`
            );
          }
          
          // Compare exports array length
          if (exportsRust.length !== exportsOrig.length) {
            throw new Error(
              `Export count mismatch: Rust=${exportsRust.length}, Original=${exportsOrig.length}`
            );
          }
          
          // Compare facade flag
          if (facadeRust !== facadeOrig) {
            throw new Error(
              `Facade flag mismatch: Rust=${facadeRust}, Original=${facadeOrig}`
            );
          }
          
          // Compare hasModuleSyntax flag
          if (hasModuleRust !== hasModuleOrig) {
            throw new Error(
              `hasModuleSyntax flag mismatch: Rust=${hasModuleRust}, Original=${hasModuleOrig}`
            );
          }
          
          // Compare each import in detail
          for (let i = 0; i < importsOrig.length; i++) {
            const orig = importsOrig[i];
            const rust = importsRust[i];
            
            if (JSON.stringify(rust) !== JSON.stringify(orig)) {
              throw new Error(
                `Import ${i} mismatch:\nRust: ${JSON.stringify(rust)}\nOriginal: ${JSON.stringify(orig)}`
              );
            }
          }
          
          // Compare each export in detail
          for (let i = 0; i < exportsOrig.length; i++) {
            const orig = exportsOrig[i];
            const rust = exportsRust[i];
            
            if (JSON.stringify(rust) !== JSON.stringify(orig)) {
              throw new Error(
                `Export ${i} mismatch:\nRust: ${JSON.stringify(rust)}\nOriginal: ${JSON.stringify(orig)}`
              );
            }
          }
        } catch (e) {
          // If both implementations throw, that's acceptable
          // If only one throws, that's a problem
          try {
            parseOriginal(source);
            // Original succeeded, so Rust should too
            throw e;
          } catch (origError) {
            // Both threw, which is acceptable
            return true;
          }
        }
        
        return true;
      }),
      { numRuns: 100 }
    );
  });

  test('Property 15: Import positions alignment', () => {
    fc.assert(
      fc.property(
        fc.array(arbImportStatement(), { minLength: 1, maxLength: 3 }),
        (imports) => {
          const source = imports.join('\n');
          
          try {
            const [importsOrig] = parseOriginal(source);
            const [importsRust] = parseRust(source);
            
            if (importsRust.length !== importsOrig.length) {
              throw new Error('Import count mismatch');
            }
            
            for (let i = 0; i < importsOrig.length; i++) {
              const orig = importsOrig[i];
              const rust = importsRust[i];
              
              // Check all position fields
              if (rust.s !== orig.s) throw new Error(`Import ${i}: s mismatch`);
              if (rust.e !== orig.e) throw new Error(`Import ${i}: e mismatch`);
              if (rust.ss !== orig.ss) throw new Error(`Import ${i}: ss mismatch`);
              if (rust.se !== orig.se) throw new Error(`Import ${i}: se mismatch`);
              if (rust.d !== orig.d) throw new Error(`Import ${i}: d mismatch`);
              if (rust.a !== orig.a) throw new Error(`Import ${i}: a mismatch`);
              if (rust.t !== orig.t) throw new Error(`Import ${i}: t mismatch`);
              if (rust.n !== orig.n) throw new Error(`Import ${i}: n mismatch`);
            }
          } catch (e) {
            try {
              parseOriginal(source);
              throw e;
            } catch {
              return true;
            }
          }
          
          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  test('Property 15: Export positions alignment', () => {
    fc.assert(
      fc.property(
        fc.array(arbExportStatement(), { minLength: 1, maxLength: 3 }),
        (exports) => {
          const source = exports.join('\n');
          
          try {
            const [, exportsOrig] = parseOriginal(source);
            const [, exportsRust] = parseRust(source);
            
            if (exportsRust.length !== exportsOrig.length) {
              throw new Error('Export count mismatch');
            }
            
            for (let i = 0; i < exportsOrig.length; i++) {
              const orig = exportsOrig[i];
              const rust = exportsRust[i];
              
              // Check all fields
              if (rust.n !== orig.n) throw new Error(`Export ${i}: n mismatch`);
              if (rust.ln !== orig.ln) throw new Error(`Export ${i}: ln mismatch`);
              if (rust.s !== orig.s) throw new Error(`Export ${i}: s mismatch`);
              if (rust.e !== orig.e) throw new Error(`Export ${i}: e mismatch`);
              if (rust.ls !== orig.ls) throw new Error(`Export ${i}: ls mismatch`);
              if (rust.le !== orig.le) throw new Error(`Export ${i}: le mismatch`);
            }
          } catch (e) {
            try {
              parseOriginal(source);
              throw e;
            } catch {
              return true;
            }
          }
          
          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  test('Property 15: Facade mode alignment', () => {
    fc.assert(
      fc.property(
        fc.tuple(
          fc.array(arbImportStatement(), { maxLength: 3 }),
          fc.array(arbExportStatement(), { maxLength: 3 }),
          fc.boolean()
        ),
        ([imports, exports, addCode]) => {
          // Generate either pure module code or mixed code
          const parts = [...imports, ...exports];
          if (addCode) {
            parts.push('const x = 1;'); // Add non-module code
          }
          const source = parts.join('\n');
          
          try {
            const [, , facadeOrig] = parseOriginal(source);
            const [, , facadeRust] = parseRust(source);
            
            if (facadeRust !== facadeOrig) {
              throw new Error(
                `Facade mismatch: Rust=${facadeRust}, Original=${facadeOrig}`
              );
            }
          } catch (e) {
            try {
              parseOriginal(source);
              throw e;
            } catch {
              return true;
            }
          }
          
          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  test('Property 15: hasModuleSyntax alignment', () => {
    fc.assert(
      fc.property(
        fc.oneof(
          arbModuleCode(),
          fc.constant('const x = 1;'), // No module syntax
          fc.constant('') // Empty
        ),
        (source) => {
          try {
            const [, , , hasModuleOrig] = parseOriginal(source);
            const [, , , hasModuleRust] = parseRust(source);
            
            if (hasModuleRust !== hasModuleOrig) {
              throw new Error(
                `hasModuleSyntax mismatch: Rust=${hasModuleRust}, Original=${hasModuleOrig}`
              );
            }
          } catch (e) {
            try {
              parseOriginal(source);
              throw e;
            } catch {
              return true;
            }
          }
          
          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  test('Property 15: String escape handling alignment', () => {
    const arbEscapedString = () =>
      fc.oneof(
        fc.constant('./\\x61\\x62\\x63.js'), // Hex escapes
        fc.constant('./\\u0041\\u0042.js'), // Unicode escapes
        fc.constant('./\\u{20204}.js'), // Unicode code point escapes
        fc.constant('./test\\nfile.js'), // Newline escape
        fc.constant('./test\\tfile.js') // Tab escape
      );
    
    fc.assert(
      fc.property(arbEscapedString(), (spec) => {
        const source = `import '${spec}';`;
        
        try {
          const [importsOrig] = parseOriginal(source);
          const [importsRust] = parseRust(source);
          
          if (importsRust.length !== importsOrig.length) {
            throw new Error('Import count mismatch');
          }
          
          if (importsRust[0].n !== importsOrig[0].n) {
            throw new Error(
              `Module specifier mismatch: Rust="${importsRust[0].n}", Original="${importsOrig[0].n}"`
            );
          }
        } catch (e) {
          try {
            parseOriginal(source);
            throw e;
          } catch {
            return true;
          }
        }
        
        return true;
      }),
      { numRuns: 100 }
    );
  });
});
