import { describe, test, expect } from 'vitest';
import { parse, ImportType } from '../src/index';

function assertExportIs(source: string, actual: any, expected: { n: string; ln?: string }) {
  if (source[actual.s] === '"' || source[actual.s] === "'") {
    expect(source[actual.s]).toBe(source[actual.e - 1]);
  } else {
    expect(source.substring(actual.s, actual.e)).toBe(expected.n);
  }
  if (expected.ln === undefined) {
    expect(actual.ls).toBe(-1);
    expect(actual.le).toBe(-1);
  } else if (source[actual.ls] === '"' || source[actual.ls] === "'") {
    expect(source[actual.ls]).toBe(source[actual.le - 1]);
  } else {
    expect(source.substring(actual.ls, actual.le)).toBe(expected.ln);
  }
  expect(actual.n).toBe(expected.n);
  expect(actual.ln).toBe(expected.ln);
}

describe('Lexer - Ported from es-module-lexer', () => {
  test('Defer phase imports', () => {
    const source = `
      import defer
        * as foo from 'specifier'
      
      import defer * as blah from './x.js' with { type: 'css' }

      import defer from 'x'

      import.defer('blah');
    
    `;
    const [impts] = parse(source);
    expect(impts.length).toBe(4);

    expect(impts[0].t).toBe(6);
    expect(source.slice(impts[0].ss, impts[0].se)).toBe(source.slice(7, 53));
    expect(source.slice(impts[0].s, impts[0].e)).toBe('specifier');
    expect(impts[0].d).toBe(-1);
    expect(impts[0].a).toBe(-1);
    expect(impts[0].at).toBe(null);

    expect(impts[1].t).toBe(6);
    expect(source.slice(impts[1].ss, impts[1].se)).toBe(`import defer * as blah from './x.js' with { type: 'css' }`);
    expect(source.slice(impts[1].s, impts[1].e)).toBe('./x.js');
    expect(impts[1].d).toBe(-1);
    expect(source.slice(impts[1].a, impts[1].se)).toBe(`{ type: 'css' }`);
    expect(impts[1].at).toEqual([['type', 'css']]);

    expect(impts[2].t).toBe(1);
    expect(source.slice(impts[2].ss, impts[2].se)).toBe(`import defer from 'x'`);
    expect(source.slice(impts[2].s, impts[2].e)).toBe("x");

    expect(impts[3].t).toBe(7);
    expect(source.slice(impts[3].ss, impts[3].se)).toBe(`import.defer('blah')`);
    expect(source.slice(impts[3].s, impts[3].e)).toBe("'blah'");
    expect(source.slice(impts[3].d, impts[3].se)).toBe(`('blah')`);
    expect(impts[3].a).toBe(-1);
  });

  test('Import attributes parsing', () => {
    const source = `
      import foo from 'module' with { type: "json" }
      import bar from 'module2' with { type: 'css', integrity: "sha384-abc" }
      import { baz } from 'module3' with { "custom-key": "value" }
      import * as ns from 'module4' with { type: "json" }
      import 'module5' with { type: "json" }
      import noAttrs from 'module6'
    `;
    const [impts] = parse(source);
    expect(impts.length).toBe(6);

    expect(impts[0].at).toEqual([['type', 'json']]);
    expect(source.slice(impts[0].s, impts[0].e)).toBe('module');

    expect(impts[1].at).toEqual([['type', 'css'], ['integrity', 'sha384-abc']]);
    expect(source.slice(impts[1].s, impts[1].e)).toBe('module2');

    expect(impts[2].at).toEqual([['custom-key', 'value']]);
    expect(source.slice(impts[2].s, impts[2].e)).toBe('module3');

    expect(impts[3].at).toEqual([['type', 'json']]);
    expect(source.slice(impts[3].s, impts[3].e)).toBe('module4');

    expect(impts[4].at).toEqual([['type', 'json']]);
    expect(source.slice(impts[4].s, impts[4].e)).toBe('module5');

    expect(impts[5].at).toBe(null);
    expect(source.slice(impts[5].s, impts[5].e)).toBe('module6');
  });

  test('Import attributes with quoted keys and escape sequences', () => {
    const source = `
      import a from 'a' with { "quoted-key": "value" }
      import b from 'b' with { 'single-quoted': "value" }
      import c from 'c' with { "key-with-\\"quote\\"": "value-with-\\"quote\\"" }
      import d from 'd' with { "key\\nwith\\nnewlines": "value\\twith\\ttabs" }
      import e from 'e' with { "unicode\\u0041": "test\\u0042" }
      import f from 'f' with { type: "val\\\\backslash" }
    `;
    const [impts] = parse(source);
    expect(impts.length).toBe(6);

    expect(impts[0].at).toEqual([['quoted-key', 'value']]);
    expect(impts[1].at).toEqual([['single-quoted', 'value']]);
    expect(impts[2].at).toEqual([['key-with-"quote"', 'value-with-"quote"']]);
    expect(impts[3].at).toEqual([['key\nwith\nnewlines', 'value\twith\ttabs']]);
    expect(impts[4].at).toEqual([['unicodeA', 'testB']]);
    expect(impts[5].at).toEqual([['type', 'val\\backslash']]);
  });

  test('Import types', () => {
    const input = `
      // dynamic
      const { a } = await import('a');
      const { b } = await import.source('b');
      // static
      import b from 'b';
      import { c } from 'c';
      import source z from 'z';
      // meta
      import.meta.url
    `;

    const [imports] = parse(input);
    expect(imports[0].t).toBe(2);
    expect(imports[1].t).toBe(5);
    expect(imports[2].t).toBe(1);
    expect(imports[3].t).toBe(1);
    expect(imports[4].t).toBe(4);
    expect(imports[5].t).toBe(3);
  });

  test('Source phase imports', () => {
    const source = `
      import source
        source from 'specifier'
      
      import source blah from './x.js' with { type: 'css' }

      import source from 'x'

      import.source('blah');
    
    `;
    const [impts] = parse(source);
    expect(impts.length).toBe(4);

    expect(impts[0].t).toBe(4);
    expect(source.slice(impts[0].ss, impts[0].se)).toBe(source.slice(7, 52));
    expect(source.slice(impts[0].s, impts[0].e)).toBe('specifier');
    expect(impts[0].d).toBe(-1);
    expect(impts[0].a).toBe(-1);

    expect(impts[1].t).toBe(4);
    expect(source.slice(impts[1].ss, impts[1].se)).toBe(`import source blah from './x.js' with { type: 'css' }`);
    expect(source.slice(impts[1].s, impts[1].e)).toBe('./x.js');
    expect(impts[1].d).toBe(-1);
    expect(source.slice(impts[1].a, impts[1].se)).toBe(`{ type: 'css' }`);

    expect(impts[2].t).toBe(1);
    expect(source.slice(impts[2].ss, impts[2].se)).toBe(`import source from 'x'`);
    expect(source.slice(impts[2].s, impts[2].e)).toBe("x");

    expect(impts[3].t).toBe(5);
    expect(source.slice(impts[3].ss, impts[3].se)).toBe(`import.source('blah')`);
    expect(source.slice(impts[3].s, impts[3].e)).toBe("'blah'");
    expect(source.slice(impts[3].d, impts[3].se)).toBe(`('blah')`);
    expect(impts[3].a).toBe(-1);
  });

  test('Dynamic import expression range', () => {
    const source = `import(("asdf"))  aaaa`;
    const [[impt]] = parse(source);
    expect(source.slice(impt.ss, impt.se)).toBe('import(("asdf"))');
    expect(source.slice(impt.s, impt.e)).toBe('("asdf")');
  });

  test('Dynamic import expression range 2', () => {
    const source = 'import(/* comment */ `asdf` /* comment */)';
    const [[impt]] = parse(source);
    expect(source.slice(impt.ss, impt.se)).toBe('import(/* comment */ `asdf` /* comment */)');
    expect(source.slice(impt.s, impt.e)).toBe('`asdf`');
  });

  test('Dynamic import expression range 3', () => {
    const source = 'import(`asdf` // comment\n)';
    const [[impt]] = parse(source);
    expect(source.slice(impt.ss, impt.se)).toBe('import(`asdf` // comment\n)');
    expect(source.slice(impt.s, impt.e)).toBe('`asdf`');
  });

  test('Dynamic import expression range 4', () => {
    const source = 'import("foo" + /* comment */ "bar")';
    const [[impt]] = parse(source);
    expect(source.slice(impt.ss, impt.se)).toBe('import("foo" + /* comment */ "bar")');
    expect(source.slice(impt.s, impt.e)).toBe('"foo" + /* comment */ "bar"');
  });

  test('Dynamic import expression range 5', () => {
    const source = 'import((() => { return "foo" })() /* comment */)';
    const [[impt]] = parse(source);
    expect(source.slice(impt.ss, impt.se)).toBe('import((() => { return "foo" })() /* comment */)');
    expect(source.slice(impt.s, impt.e)).toBe('(() => { return "foo" })()');
  });

  test('Simple export destructuring', () => {
    const source = `
      export const{URI,Utils,...Another}=LIB
      export var p, { z } = {};

      export var { aa, qq: { z } } = { qq: {} }, pp = {};
    `;
    const [, exports] = parse(source);
    expect(exports.map(e => e.n)).toEqual(['URI', 'Utils', 'p', 'aa', 'qq']);
  });

  test('Export default cases', () => {
    const source = `
      export default "export default a"
      export default "export default 'a'"
      export default "export function foo() {}"
      export default "export function foo() {return bar}"
    `;
    const [, exports] = parse(source);
    expect(exports.map(expt => expt.n)).toEqual(['default', 'default', 'default', 'default']);
  });

  test('import.meta spread', () => {
    const source = `console.log(...import.meta.obj);`;
    const [impts] = parse(source);
    expect(impts.length).toBe(1);
    expect(source.substring(impts[0].s, impts[0].e)).toBe('import.meta');
  });

  test('Template string default bracket', () => {
    const source = `export default{};`;
    const [, [expt]] = parse(source);
    expect(source.slice(expt.s, expt.e)).toBe('default');
    expect(source.slice(expt.ls, expt.le)).toBe('');
    expect(expt.n).toBe('default');
    expect(expt.ln).toBe(undefined);
  });

  test('Template string default', () => {
    const source = `const css = String.raw;
        export default css\`:host { solid 1px black }\`;`;
    const [, [expt]] = parse(source);
    expect(source.slice(expt.s, expt.e)).toBe('default');
    expect(source.slice(expt.ls, expt.le)).toBe('');
    expect(expt.n).toBe('default');
    expect(expt.ln).toBe(undefined);
  });

  test('Class fn ASI', () => {
    parse(`class a{friendlyName;import}n();`);
  });

  test('Division const after class parse case', () => {
    const source = `class a{}const Ti=a/yi;`;
    parse(source);
  });

  test('Basic nested dynamic import support', () => {
    const source = `await import (await import  ('foo'))`;
    const [imports] = parse(source);
    expect(imports.length).toBe(2);
    expect(source.slice(imports[0].ss, imports[0].d)).toBe('import ');
    expect(source.slice(imports[0].ss, imports[0].se)).toBe('import (await import  (\'foo\'))');
    expect(source.slice(imports[0].s, imports[0].e)).toBe('await import  (\'foo\')');
    expect(source.slice(imports[1].ss, imports[1].d)).toBe('import  ');
    expect(source.slice(imports[1].ss, imports[1].se)).toBe('import  (\'foo\')');
    expect(source.slice(imports[1].s, imports[1].e)).toBe('\'foo\'');
  });

  test('Import meta inside dynamic import', () => {
    const source = `import(import.meta.url)`;
    const [imports] = parse(source);

    expect(imports.length).toBe(2);
    expect(source.substring(imports[0].s, imports[0].e)).toBe('import.meta.url');
  });

  test('Export', () => {
    const source = `export var p=5`;
    const [, exports] = parse(source);
    assertExportIs(source, exports[0], { n: 'p', ln: 'p' });
  });

  test('String encoding', () => {
    const [imports,] = parse(`
      import './\\x61\\x62\\x63.js';
      import './\\u{20204}.js';
      import('./\\u{20204}.js');
      import('./\\u{20204}.js' + dyn);
      import('./\\u{20204}.js' );
      import('./\\u{20204}.js' ());
    `);
    expect(imports.length).toBe(6);
    expect(imports[0].n).toBe('./abc.js');
    expect(imports[1].n).toBe('./𠈄.js');
    expect(imports[2].n).toBe('./𠈄.js');
    expect(imports[3].n).toBe(undefined);
    expect(imports[4].n).toBe('./𠈄.js');
    expect(imports[5].n).toBe(undefined);
  });

  test('Regexp case', () => {
    parse(`
      class Number {

      }

      /("|')(?<value>(\\\\(\\1)|[^\\1])*)?(\\1)/.exec(\`'\\\\"\\\\'aa'\`);

      const x = \`"\${label.replace(/"/g, "\\\\\\"")}"\`
    `);
  });

  test('Regexp default export', () => {
    const source = `
      export default /[\`]/
      export default 1/2
      export default /* asdf */ 1/2
      export default /* asdf */ /regex/
      export default
      // line comment
      /regex/
      export default
      // line comment
      1 / 2
    `;
    const [, exports] = parse(source);
    expect(exports.map(expt => expt.n)).toEqual(['default', 'default', 'default', 'default', 'default', 'default']);
  });

  test('Regexp division', () => {
    parse(`\nconst x = num / /'/.exec(l)[0].slice(1, -1)//'"`);
  });

  test('Multiline string escapes', () => {
    parse("const str = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAB4AAAAeCAYAAAA7MK6iAAAABmJLR0QA/wAAAAAzJ3zzAAAGTElEQV\\\r\n\t\tRIx+VXe1BU1xn/zjn7ugvL4sIuQnll5U0ELAQxig7WiQYz6NRHa6O206qdSXXSxs60dTK200zNY9q0dcRpMs1jkrRNWmaijCVoaU';\r\n");
  });

  test('Dotted number', () => {
    parse(`
       const x = 5. / 10;
    `);
  });

  test('Division operator case', () => {
    parse(`
      function log(r){
        if(g>=0){u[g++]=m;g>=n.logSz&&(g=0)}else{u.push(m);u.length>=n.logSz&&(g=0)}/^(DBG|TICK): /.test(r)||t.Ticker.tick(454,o.slice(0,200));
      }

      (function(n){
      })();
    `);
  });

  test('Single parse cases', () => {
    parse(`export { x }`);
    parse(`'asdf'`);
    parse(`/asdf/`);
    parse(`\`asdf\``);
    parse(`/**/`);
    parse(`//`);
  });

  test('Simple export with unicode conversions', () => {
    const source = `export var p𓀀s,q`;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(0);
    expect(exports.length).toBe(2);
    assertExportIs(source, exports[0], {n: 'p𓀀s', ln: 'p𓀀s' });
    assertExportIs(source, exports[1], {n: 'q', ln: 'q' });
  });

  test('Simple import', () => {
    const source = `
      import test from "test";
      console.log(test);
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(1);
    const { s, e, ss, se, d, n } = imports[0];
    expect(d).toBe(-1);
    expect(n).toBe('test');
    expect(source.slice(ss, se)).toBe('import test from "test"');
    expect(exports.length).toBe(0);
  });

  test('Empty single quote string import', () => {
    const source = `import ''`;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(1);
    const { s, e, ss, se, d } = imports[0];
    expect(d).toBe(-1);
    expect(source.slice(s, e)).toBe('');
    expect(source.slice(ss, se)).toBe(`import ''`);
    expect(exports.length).toBe(0);
  });

  test('Empty double quote string import', () => {
    const source = `import ""`;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(1);
    const { s, e, ss, se, d } = imports[0];
    expect(d).toBe(-1);
    expect(source.slice(s, e)).toBe('');
    expect(source.slice(ss, se)).toBe('import ""');
    expect(exports.length).toBe(0);
  });

  test('Import/Export with comments', () => {
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
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(2);
    expect(source.slice(imports[0].s, imports[0].e)).toBe('a');
    expect(source.slice(imports[0].ss, imports[0].se)).toBe(`import/* 'x' */ 'a'`);
    expect(source.slice(imports[1].s, imports[1].e)).toBe('b');
    expect(source.slice(imports[1].ss, imports[1].se)).toBe(`import /* 'x' */ 'b'`);
    expect(exports.length).toBe(3);
    assertExportIs(source, exports[0], { n: 'z', ln: 'z' });
    assertExportIs(source, exports[1], { n: 'a', ln: 'a' });
    assertExportIs(source, exports[2], { n: 'd', ln: 'd' });
  });

  test('Exported function and class', () => {
    const source = `
      export function a𓀀 () {

      }
      export class Q{

      }
    `;
    const [, exports] = parse(source);
    expect(exports.length).toBe(2);
    assertExportIs(source, exports[0], {n: 'a𓀀', ln: 'a𓀀' });
    assertExportIs(source, exports[1], {n: 'Q', ln: 'Q' });
  });

  test('Export destructuring', () => {
    const source = `
      export const { a, b } = foo;

      export { ok };
    `;
    const [, exports] = parse(source);
    expect(exports.length).toBe(3);
    assertExportIs(source, exports[0], { n: 'a', ln: 'a' });
  });

  test('Minified import syntax', () => {
    const source = `import{TemplateResult as t}from"lit-html";import{a as e}from"./chunk-4be41b30.js";export{j as SVGTemplateResult,i as TemplateResult,g as html,h as svg}from"./chunk-4be41b30.js";window.JSCompiler_renameProperty='asdf';`;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(3);
    expect(imports[0].s).toBe(32);
    expect(imports[0].e).toBe(40);
    expect(imports[0].ss).toBe(0);
    expect(imports[0].se).toBe(41);
    expect(imports[1].s).toBe(61);
    expect(imports[1].e).toBe(80);
    expect(imports[1].ss).toBe(42);
    expect(imports[1].se).toBe(81);
    expect(imports[2].s).toBe(156);
    expect(imports[2].e).toBe(175);
    expect(imports[2].ss).toBe(82);
    expect(imports[2].se).toBe(176);
  });

  test('More minified imports', () => {
    const source = `import"some/import.js";`
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(1);
    expect(imports[0].s).toBe(7);
    expect(imports[0].e).toBe(21);
    expect(imports[0].ss).toBe(0);
    expect(imports[0].se).toBe(22);
  });

  test('plus plus division', () => {
    parse(`
      tick++/fetti;f=(1)+")";
    `);
  });

  test('return bracket division', () => {
    const source = `function variance(){return s/(a-1)}`;
    parse(source);
  });

  test('Simple reexport', () => {
    const source = `
      export { hello as default } from "test-dep";
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(1);
    const { s, e, ss, se, d } = imports[0];
    expect(d).toBe(-1);
    expect(source.slice(s, e)).toBe('test-dep');
    expect(source.slice(ss, se)).toBe('export { hello as default } from "test-dep"');

    expect(exports.length).toBe(1);
    assertExportIs(source, exports[0], { n: 'default', ln: undefined });
  });

  test('import.meta', () => {
    const source = `
      export var hello = 'world';
      console.log(import.meta.url);
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(1);
    const { s, e, ss, se, d } = imports[0];
    expect(d).toBe(-2);
    expect(ss).toBe(53);
    expect(se).toBe(64);
    expect(source.slice(s, e)).toBe('import.meta');
  });

  test('import meta edge cases', () => {
    const source = `
      // Import meta
      import.
       meta
      // Not import meta
      a.
      import.
        meta
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(1);
    const { s, e, ss, se, d } = imports[0];
    expect(d).toBe(-2);
    expect(ss).toBe(28);
    expect(se).toBe(47);
    expect(source.slice(s, e)).toBe('import.\n       meta');
  });

  test('dynamic import method', () => {
    const source = `
      class A {
        import() {
        }
      }
    `;
    const [imports] = parse(source);
    expect(imports.length).toBe(0);
  });

  test('dynamic import edge cases', () => {
    const source = `
      ({
        // not a dynamic import!
        import(not1) {}
      });
      {
        // is a dynamic import!
        import(is1);
      }
      a.
      // not a dynamic import!
      import(not2);
      a.
      b()
      // is a dynamic import!
      import(is2);

      const myObject = {
        import: ()=> import(some_url)
      }
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(3);
    let { s, e, ss, se, d } = imports[0];
    expect(ss + 6).toBe(d);
    expect(se).toBe(e + 1);
    expect(source.slice(d, se)).toBe('(is1)');
    expect(source.slice(s, e)).toBe('is1');

    ({ s, e, ss, se, d } = imports[1]);
    expect(ss + 6).toBe(d);
    expect(se).toBe(e + 1);
    expect(source.slice(s, e)).toBe('is2');

    ({ s, e, ss, se, d } = imports[2]);
    expect(ss + 6).toBe(d);
    expect(se).toBe(e + 1);
    expect(source.slice(s, e)).toBe('some_url');
  });

  test('import after code', () => {
    const source = `
      export function f () {
        g();
      }

      import { g } from './test-circular2.js';
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(1);
    const { s, e, ss, se, d } = imports[0];
    expect(d).toBe(-1);
    expect(source.slice(s, e)).toBe('./test-circular2.js');
    expect(source.slice(ss, se)).toBe(`import { g } from './test-circular2.js'`);
    expect(exports.length).toBe(1);
    assertExportIs(source, exports[0], { n: 'f', ln: 'f' });
  });

  test('Comments', () => {
    const source = `/*
    VERSION
  */import util from 'util';

//
function x() {
}

      /**/
      // '
      /* / */
      /*

         * export { b }
      \\*/export { a }

      function () {
        /***/
      }
    `
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(1);
    expect(source.slice(imports[0].s, imports[0].e)).toBe('util');
    expect(source.slice(imports[0].ss, imports[0].se)).toBe(`import util from 'util'`);
    expect(exports.length).toBe(1);
    assertExportIs(source, exports[0], { n: 'a', ln: 'a' });
  });

  test('Strings', () => {
    const source = `
      "";
      \`
        \${
          import(\`test/\${ import(b)}\`); /*
              \`  }
          */
        }
      \`
      export { a }
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(2);
    expect(imports[0].d).not.toBe(-1);
    expect(imports[0].ss + 6).toBe(imports[0].d);
    expect(imports[0].se).toBe(imports[0].e + 1);
    expect(source.slice(imports[0].ss, imports[0].s)).toBe('import(');
    expect(imports[1].d).not.toBe(-1);
    expect(imports[1].ss + 6).toBe(imports[1].d);
    expect(imports[1].se).toBe(imports[1].e + 1);
    expect(source.slice(imports[1].ss, imports[1].d)).toBe('import');
    expect(exports.length).toBe(1);
    assertExportIs(source, exports[0], { n: 'a', ln: 'a' });
  });

  test('Bracket matching', () => {
    parse(`
      instance.extend('parseExprAtom', function (nextMethod) {
        return function () {
          function parseExprAtom(refDestructuringErrors) {
            if (this.type === tt._import) {
              return parseDynamicImport.call(this);
            }
            return c(refDestructuringErrors);
          }
        }();
      });
      export { a }
    `);
  });

  test('Division / Regex ambiguity', () => {
    const source = `
      /as)df/; x();
      a / 2; '  /  '
      while (true)
        /test'/
      x-/a'/g
      try {}
      finally{}/a'/g
      (x);{f()}/d'export { b }/g
      ;{}/e'/g;
      {}/f'/g
      a / 'b' / c;
      /a'/ - /b'/;
      +{} /g -'/g'
      ('a')/h -'/g'
      if //x
      ('a')/i'/g;
      /asdf/ / /as'df/; // '
      p = \`\${/test/ + 5}\`;
      /regex/ / x;
      function m() {
        return /*asdf8*// 5/;
      }
      export { a };
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(0);
    expect(exports.length).toBe(1);
    assertExportIs(source, exports[0], { n: 'a', ln: 'a' });
  });

  test('Template string expression ambiguity', () => {
    const source = `
      \`$\`
      import 'a';
      \`\`
      export { b };
      \`a$b\`
      import(\`$\`);
      \`{$}\`
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(2);
    expect(exports.length).toBe(1);
    assertExportIs(source, exports[0], { n: 'b', ln: 'b' });
  });

  test('Empty export', () => {
    const source = `
      export {};
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(0);
    expect(exports.length).toBe(0);
  });

  test('Export * as', () => {
    const source = `
      export * as X from './asdf';
      export *  as  yy from './g';
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(2);
    expect(exports.length).toBe(2);
    assertExportIs(source, exports[0], { n: 'X', ln: undefined });
    assertExportIs(source, exports[1], { n: 'yy', ln: undefined });
  });

  test('non-identifier-string as (doubleQuote)', () => {
    const source = `
      import { "~123" as foo0 } from './mod0.js';
      import { "ab cd" as foo1 } from './mod1.js';
      import { "not identifier" as foo2 } from './mod2.js';
    `;
    const [imports, exports] = parse(source);
    expect(exports.length).toBe(0);
    expect(imports.length).toBe(3);

    expect(imports[0].n).toBe('./mod0.js');
    expect(imports[1].n).toBe('./mod1.js');
    expect(imports[2].n).toBe('./mod2.js');
  });

  test('Export From - Identifier only', () => {
    const source = `
      export { x } from './asdf';
      export { x1, x2 } from './g';
      export { foo, x2 as bar, zoo } from './g2';
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(3);
    expect(exports.length).toBe(6);
    assertExportIs(source, exports[0], { n: 'x', ln: undefined });
    assertExportIs(source, exports[1], { n: 'x1', ln: undefined });
    assertExportIs(source, exports[2], { n: 'x2', ln: undefined });
    assertExportIs(source, exports[3], { n: 'foo', ln: undefined });
    assertExportIs(source, exports[4], { n: 'bar', ln: undefined });
    assertExportIs(source, exports[5], { n: 'zoo', ln: undefined });
  });

  test('Export From - non-identifier-string as variable (doubleQuote)', () => {
    const source = `
      export { "~123" as foo0 } from './mod0.js';
      export { "ab cd" as foo1 } from './mod1.js';
      export { "not identifier" as foo2 } from './mod2.js';
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(3);

    expect(exports.length).toBe(3);
    assertExportIs(source, exports[0], { n: 'foo0', ln: undefined });
    assertExportIs(source, exports[1], { n: 'foo1', ln: undefined });
    assertExportIs(source, exports[2], { n: 'foo2', ln: undefined });
  });

  test('Export From - variable as non-identifier-string (doubleQuote)', () => {
    const source = `
      export { foo0 as "~123" } from './mod0.js';
      export { foo1 as "ab cd" } from './mod1.js';
      export { foo2 as "not identifier" } from './mod2.js';
    `;
    const [imports, exports] = parse(source);
    expect(imports.length).toBe(3);

    expect(exports.length).toBe(3);
    assertExportIs(source, exports[0], { n: '~123', ln: undefined });
    assertExportIs(source, exports[1], { n: 'ab cd', ln: undefined });
    assertExportIs(source, exports[2], { n: 'not identifier', ln: undefined });
  });

  test('Facade detection - pure module', () => {
    const source = `
      import foo from 'bar';
      export const x = 1;
    `;
    const [imports, exports, facade] = parse(source);
    expect(facade).toBe(true);
  });

  test('Facade detection - mixed code', () => {
    const source = `
      import foo from 'bar';
      const x = 1;
      export { x };
    `;
    const [imports, exports, facade] = parse(source);
    expect(facade).toBe(false);
  });

  test('hasModuleSyntax - with imports', () => {
    const source = `import foo from 'bar';`;
    const [, , , hasModuleSyntax] = parse(source);
    expect(hasModuleSyntax).toBe(true);
  });

  test('hasModuleSyntax - with exports', () => {
    const source = `export const x = 1;`;
    const [, , , hasModuleSyntax] = parse(source);
    expect(hasModuleSyntax).toBe(true);
  });

  test('hasModuleSyntax - no module syntax', () => {
    const source = `const x = 1;`;
    const [, , , hasModuleSyntax] = parse(source);
    expect(hasModuleSyntax).toBe(false);
  });
});
