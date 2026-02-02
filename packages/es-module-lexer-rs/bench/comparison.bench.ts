import { bench, describe } from 'vitest';
import { readFileSync } from 'fs';
import { join } from 'path';
import { parse as parseRust } from '../index.js';

// Import the original es-module-lexer for comparison
let parseOriginal: any;
try {
  parseOriginal = require('es-module-lexer').parse;
} catch (e) {
  console.warn('es-module-lexer not installed, skipping comparison benchmarks');
}

describe('Simple cases', () => {
  const simpleImport = `import foo from 'bar';`;
  
  if (parseOriginal) {
    bench('original: simple import', () => {
      parseOriginal(simpleImport);
    });
  }
  
  bench('rust: simple import', () => {
    parseRust(simpleImport);
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
  
  bench('rust: multiple imports', () => {
    parseRust(multipleImports);
  });
});

describe('Dynamic imports', () => {
  const dynamicImports = `
    import('dynamic1');
    import('dynamic2');
    const mod = import('dynamic3');
    async function load() {
      return await import('dynamic4');
    }
  `;
  
  if (parseOriginal) {
    bench('original: dynamic imports', () => {
      parseOriginal(dynamicImports);
    });
  }
  
  bench('rust: dynamic imports', () => {
    parseRust(dynamicImports);
  });
});

describe('Exports', () => {
  const exports = `
    export const x = 1;
    export function foo() {}
    export default class Bar {}
    export { a, b as c } from 'module';
    export * from 'namespace';
  `;
  
  if (parseOriginal) {
    bench('original: exports', () => {
      parseOriginal(exports);
    });
  }
  
  bench('rust: exports', () => {
    parseRust(exports);
  });
});

describe('Import attributes', () => {
  const importAttributes = `
    import data from './data.json' with { type: 'json' };
    import styles from './styles.css' with { type: 'css' };
  `;
  
  if (parseOriginal) {
    bench('original: import attributes', () => {
      parseOriginal(importAttributes);
    });
  }
  
  bench('rust: import attributes', () => {
    parseRust(importAttributes);
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
  
  bench('rust: complex module', () => {
    parseRust(complexModule);
  });
});

describe('Edge cases', () => {
  const regexVsDivision = `
    const regex = /import\\s+from/;
    const division = 10 / 2;
  `;
  
  if (parseOriginal) {
    bench('original: regex vs division', () => {
      parseOriginal(regexVsDivision);
    });
  }
  
  bench('rust: regex vs division', () => {
    parseRust(regexVsDivision);
  });
  
  const templateStrings = `
    const url = \`\${import.meta.url}\`;
    const nested = \`outer \${\`inner \${x}\`}\`;
  `;
  
  if (parseOriginal) {
    bench('original: template strings', () => {
      parseOriginal(templateStrings);
    });
  }
  
  bench('rust: template strings', () => {
    parseRust(templateStrings);
  });
  
  const comments = `
    // Single line comment
    /* Multi-line
       comment */
    import foo from 'bar'; // inline comment
    /* import 'fake'; */ // commented import
  `;
  
  if (parseOriginal) {
    bench('original: comments', () => {
      parseOriginal(comments);
    });
  }
  
  bench('rust: comments', () => {
    parseRust(comments);
  });
});
