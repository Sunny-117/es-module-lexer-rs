import { describe, it, expect } from 'vitest';
import { parse } from '../src/index.js';

describe('WASM Parser', () => {
  it('should parse simple import', async () => {
    const source = `import foo from 'bar';`;
    const result = await parse(source);
    
    expect(result.imports).toHaveLength(1);
    expect(result.imports[0].n).toBe('bar');
    expect(result.exports).toHaveLength(0);
    expect(result.hasModuleSyntax).toBe(true);
  });

  it('should parse complex module', async () => {
    const source = `
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
    
    const result = await parse(source);
    
    expect(result.imports.length).toBeGreaterThan(0);
    expect(result.exports.length).toBeGreaterThan(0);
    expect(result.hasModuleSyntax).toBe(true);
  });
});
