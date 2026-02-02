import { bench, describe } from 'vitest';
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';
import { parse as parseRust } from '../index.js';

// Import the original es-module-lexer for comparison
let parseOriginal: any;
try {
  parseOriginal = require('es-module-lexer').parse;
} catch (e) {
  console.warn('es-module-lexer not installed, skipping comparison benchmarks');
}

// Helper to load sample files
function loadSample(filename: string): string | null {
  const paths = [
    join(__dirname, '../../../es-module-lexer/test/samples', filename),
    join(process.cwd(), 'es-module-lexer/test/samples', filename),
  ];
  
  for (const path of paths) {
    if (existsSync(path)) {
      return readFileSync(path, 'utf-8');
    }
  }
  
  console.warn(`Sample file not found: ${filename}`);
  return null;
}

// Test files of various sizes
const testFiles = [
  { name: 'magic-string.js', file: 'magic-string.js' },
  { name: 'magic-string.min.js', file: 'magic-string.min.js' },
  { name: 'd3.js', file: 'd3.js' },
  { name: 'd3.min.js', file: 'd3.min.js' },
  { name: 'rollup.js', file: 'rollup.js' },
  { name: 'rollup.min.js', file: 'rollup.min.js' },
  { name: 'angular.js', file: 'angular.js' },
  { name: 'angular.min.js', file: 'angular.min.js' },
];

for (const { name, file } of testFiles) {
  const source = loadSample(file);
  
  if (source) {
    describe(`Real-world: ${name} (${(source.length / 1024).toFixed(1)}KB)`, () => {
      if (parseOriginal) {
        bench('original', () => {
          parseOriginal(source);
        });
      }
      
      bench('rust', () => {
        parseRust(source);
      });
    });
  }
}

// Synthetic file size tests
describe('File sizes', () => {
  const sizes = [
    { name: '1KB', size: 1024 },
    { name: '10KB', size: 10 * 1024 },
    { name: '100KB', size: 100 * 1024 },
    { name: '1MB', size: 1024 * 1024 },
  ];
  
  for (const { name, size } of sizes) {
    // Generate synthetic module with imports/exports
    let source = '';
    const importLine = "import foo from 'bar';\n";
    const exportLine = "export const x = 1;\n";
    const commentLine = "// This is a comment line to fill space\n";
    
    while (source.length < size) {
      source += importLine;
      source += exportLine;
      source += commentLine;
    }
    
    describe(`Synthetic: ${name}`, () => {
      if (parseOriginal) {
        bench('original', () => {
          parseOriginal(source);
        });
      }
      
      bench('rust', () => {
        parseRust(source);
      });
    });
  }
});

// Facade vs Full mode
describe('Parsing modes', () => {
  const facadeSource = `
    import foo from 'bar';
    import { a, b } from 'module';
    export const x = 1;
    export function test() {}
  `;
  
  const fullSource = `
    import foo from 'bar';
    const x = 1;
    function test() {
      return x + 1;
    }
    export { test };
  `;
  
  describe('Facade mode (pure module)', () => {
    if (parseOriginal) {
      bench('original', () => {
        parseOriginal(facadeSource);
      });
    }
    
    bench('rust', () => {
      parseRust(facadeSource);
    });
  });
  
  describe('Full mode (mixed code)', () => {
    if (parseOriginal) {
      bench('original', () => {
        parseOriginal(fullSource);
      });
    }
    
    bench('rust', () => {
      parseRust(fullSource);
    });
  });
});
