/**
 * es-module-lexer-rs
 * 
 * Rust implementation of es-module-lexer with Node.js bindings.
 * 
 * This package provides a fast, Rust-based implementation of es-module-lexer
 * with full API compatibility with the original JavaScript/WebAssembly version.
 */

// Import the native binding
// @ts-ignore - Native module path
import { parse as nativeParse } from '../index.js';

/**
 * Import specifier interface.
 *
 * Represents a single import statement or expression in the source code.
 */
export interface ImportSpecifier {
  /** Module specifier string (only if safe string literal) */
  n?: string;
  /** Import type (see ImportType enum) */
  t: number;
  /** Module specifier start position (UTF-16 code units) */
  s: number;
  /** Module specifier end position (UTF-16 code units) */
  e: number;
  /** Statement start position (UTF-16 code units) */
  ss: number;
  /** Statement end position (UTF-16 code units) */
  se: number;
  /** Dynamic import position, -1 if static (UTF-16 code units) */
  d: number;
  /** Attributes start position, -1 if none (UTF-16 code units) */
  a: number;
  /** Import attributes as array of [key, value] pairs, or null if none */
  at: string[][] | null;
}

/**
 * Export specifier interface.
 * 
 * Represents a single export statement in the source code.
 */
export interface ExportSpecifier {
  /** Export name */
  n: string;
  /** Local name (if different from export name) */
  ln?: string;
  /** Export name start position (UTF-16 code units) */
  s: number;
  /** Export name end position (UTF-16 code units) */
  e: number;
  /** Local name start position, -1 if none (UTF-16 code units) */
  ls: number;
  /** Local name end position, -1 if none (UTF-16 code units) */
  le: number;
}

/**
 * Import type enumeration.
 * 
 * Distinguishes between different kinds of import statements and expressions.
 */
export enum ImportType {
  /** Static import: `import foo from 'bar'` */
  Static = 1,
  /** Dynamic import: `import('foo')` */
  Dynamic = 2,
  /** Import meta: `import.meta` */
  ImportMeta = 3,
  /** Static source phase: `import source foo from 'bar'` */
  StaticSourcePhase = 4,
  /** Dynamic source phase: `import.source('foo')` */
  DynamicSourcePhase = 5,
  /** Static defer phase: `import defer foo from 'bar'` */
  StaticDeferPhase = 6,
  /** Dynamic defer phase: `import.defer('foo')` */
  DynamicDeferPhase = 7,
}

/**
 * Parse JavaScript source code to extract imports and exports.
 * 
 * This function analyzes JavaScript/TypeScript module code and extracts all
 * import and export statements, along with their positions and metadata.
 * 
 * @param source - JavaScript source code to parse
 * @param name - Optional module name for error messages (currently unused)
 * @returns A tuple containing:
 *   - imports: Array of import specifiers
 *   - exports: Array of export specifiers
 *   - facade: Whether this is a facade module (pure imports/exports only)
 *   - hasModuleSyntax: Whether the file contains any module syntax
 * 
 * @example
 * ```typescript
 * import { parse } from 'es-module-lexer-rs';
 * 
 * const source = `
 *   import foo from 'bar';
 *   export const x = 1;
 * `;
 * 
 * const [imports, exports, facade, hasModuleSyntax] = parse(source);
 * console.log(imports[0].n); // 'bar'
 * console.log(exports[0].n); // 'x'
 * ```
 */
export function parse(
  source: string,
  name?: string
): readonly [
  imports: ReadonlyArray<ImportSpecifier>,
  exports: ReadonlyArray<ExportSpecifier>,
  facade: boolean,
  hasModuleSyntax: boolean
] {
  const result = nativeParse(source, name);
  return [result.imports, result.exports, result.facade, result.hasModuleSyntax] as const;
}
