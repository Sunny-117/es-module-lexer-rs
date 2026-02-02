import { bench, describe } from 'vitest';
import { parse as parseWasm } from '../src/index.js';

// Import the original es-module-lexer for comparison
let parseOriginal: any;
try {
  parseOriginal = require('es-module-lexer').parse;
} catch (e) {
  console.warn('es-module-lexer not installed, skipping comparison benchmarks');
}

// Initialize WASM before benchmarks
await parseWasm('');

describe('Simple cases', () => {
  const simpleImport = `import foo from 'bar';`;
  
  if (parseOriginal) {
    bench('original: simple import', () => {
      parseOriginal(simpleImport);
    });
  }
  
  bench('wasm: simple import', async () => {
    await parseWasm(simpleImport);
  });
});

describe('Multiple imports', () => {
  const multipleImports = `
    import foo from 'bar';
    import { a, b, c } from 'module';
    import * as ns from 'namespace';
    import def, { named } from 'mixed';
  `;
  
  if (parseOriginal) {
    bench('original: multiple imports', () => {
      parseOriginal(multipleImports);
    });
  }
  
  bench('wasm: multiple imports', async () => {
    await parseWasm(multipleImports);
  });
});

describe('Complex module', () => {
  const complexModule = `
    import foo from 'bar';
    import { a, b, c } from 'module';
    import * as ns from 'namespace';
    import('dynamic');
    
    export const x = 1;
    export function test() {
      const regex = /import\\s+from/;
      const str = "import 'fake'";
      return \`template \${import.meta.url}\`;
    }
    
    export default class MyClass {
      method() {
        import('lazy').then(m => m.default);
      }
    }
  `;
  
  if (parseOriginal) {
    bench('original: complex module', () => {
      parseOriginal(complexModule);
    });
  }
  
  bench('wasm: complex module', async () => {
    await parseWasm(complexModule);
  });
});
