# ES Module Lexer (Rust)

> ⚠️ **Work In Progress** - This project is under active development. Do not use in production environments.

🦀 A Rust implementation of [es-module-lexer](https://github.com/guybedford/es-module-lexer) with Node.js bindings via napi-rs.

Fast JavaScript ES module lexer that outputs the list of exports and locations of import specifiers, including dynamic import and import meta handling. Built with Rust for memory safety and maintainability.

## Why Rust?

This library provides a **memory-safe, maintainable alternative** to the original es-module-lexer:

- 🔒 **Memory Safety**: Rust's ownership system prevents segfaults and memory leaks
- 📘 **Type Safety**: Full TypeScript definitions with compile-time guarantees  
- 🛠️ **Maintainability**: Modern Rust codebase with cargo, clippy, rustfmt
- 🎯 **API Compatible**: Drop-in replacement for es-module-lexer
- 🧪 **Well Tested**: Unit tests, integration tests, and property-based tests

## Features

- ✅ Static and dynamic import/export parsing
- ✅ Import attributes (with syntax)
- ✅ Source phase imports
- ✅ Import meta detection
- ✅ Facade module detection
- ✅ Cross-platform pre-built binaries
- ✅ Zero runtime dependencies

## Installation

```bash
npm install es-module-lexer-rs
```

Pre-built binaries are provided for Linux, macOS, Windows, and FreeBSD (x64, ARM64).

## API

```typescript
interface ImportSpecifier {
  n?: string;      // module specifier
  t: number;       // import type (1=static, 2=dynamic, 3=import.meta, etc.)
  s: number;       // module specifier start
  e: number;       // module specifier end
  ss: number;      // statement start
  se: number;      // statement end
  d: number;       // dynamic import position (-1 if not dynamic)
  a: number;       // assert/with clause position (-1 if none)
  at?: [string, string][] | null;  // parsed import attributes
}

interface ExportSpecifier {
  n: string;       // export name
  ln?: string;     // local name (for re-exports)
  s: number;       // export name start
  e: number;       // export name end
  ls: number;      // local name start (-1 if none)
  le: number;      // local name end (-1 if none)
}

interface ParseResult {
  imports: ImportSpecifier[];
  exports: ExportSpecifier[];
  facade: boolean;          // true if module only contains imports/exports
  hasModuleSyntax: boolean; // true if module has any import/export
}

function parse(source: string): ParseResult;
```

For detailed usage examples, see the [documentation](./docs/).

## Supported Syntax

- Static and dynamic imports/exports
- Import attributes (`with` syntax)
- Source phase imports
- Import meta
- String export names
- Re-exports
- All ES module syntax variations

## Documentation

- [Performance Analysis](./docs/performance-analysis.md)
- [Architecture](./docs/architecture.md)
- [Contributing Guide](./CONTRIBUTING.md)

## License

MIT

## Acknowledgments

Inspired by [es-module-lexer](https://github.com/guybedford/es-module-lexer) by Guy Bedford.

---

**Built with Rust 🦀 and napi-rs**
