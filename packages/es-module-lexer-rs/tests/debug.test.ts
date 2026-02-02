import { test } from 'vitest';
import { parse as parseRust } from '../src/index';
import { parse as parseOriginal, init } from 'es-module-lexer';

await init;

test('debug simple import', () => {
  const source = `import foo from 'bar';`;
  
  const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
  const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
  
  console.log('Original imports:', JSON.stringify(importsOrig, null, 2));
  console.log('Rust imports:', JSON.stringify(importsRust, null, 2));
  
  console.log('Original exports:', JSON.stringify(exportsOrig, null, 2));
  console.log('Rust exports:', JSON.stringify(exportsRust, null, 2));
  
  console.log('Original facade:', facadeOrig, 'hasModule:', hasModuleOrig);
  console.log('Rust facade:', facadeRust, 'hasModule:', hasModuleRust);
});
