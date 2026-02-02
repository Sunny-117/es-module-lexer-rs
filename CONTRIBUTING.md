# Contributing to es-module-lexer-rs

Thank you for your interest in contributing to es-module-lexer-rs! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Environment Setup](#development-environment-setup)
4. [Project Structure](#project-structure)
5. [Development Workflow](#development-workflow)
6. [Testing](#testing)
7. [Benchmarking](#benchmarking)
8. [Code Style](#code-style)
9. [Commit Guidelines](#commit-guidelines)
10. [Pull Request Process](#pull-request-process)
11. [Areas for Contribution](#areas-for-contribution)

## Code of Conduct

This project follows a simple code of conduct:

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Prioritize safety and correctness over performance

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- **Rust**: 1.70 or higher ([Install from rustup.rs](https://rustup.rs/))
- **Node.js**: 18 or higher
- **pnpm**: 8 or higher (`npm install -g pnpm`)
- **Git**: For version control

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/es-module-lexer-rs.git
   cd es-module-lexer-rs
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/Sunny-117/es-module-lexer-rs.git
   ```

## Development Environment Setup

### Initial Setup

```bash
# Install dependencies
pnpm install

# Build Rust core library
cargo build --package es-module-lexer

# Build Node.js bindings
cd packages/es-module-lexer-rs
pnpm build:rust

# Build TypeScript
pnpm build:ts

# Or build everything at once
pnpm build
```

### Verify Setup

```bash
# Run tests to verify everything works
pnpm test

# Run a simple benchmark
pnpm bench
```

## Project Structure

```
es-module-lexer-rs/
├── crates/
│   ├── es-module-lexer/          # Core Rust lexer
│   │   ├── src/
│   │   │   ├── lib.rs            # Public API
│   │   │   ├── lexer.rs          # Main lexer implementation
│   │   │   ├── types.rs          # Data structures
│   │   │   ├── error.rs          # Error types
│   │   │   ├── parser/           # Import/export parsing
│   │   │   └── scanner/          # Token scanning
│   │   └── Cargo.toml
│   └── es-module-lexer-wasm/     # WASM bindings
├── packages/
│   └── es-module-lexer-rs/       # Node.js package
│       ├── native/               # Napi-rs bindings
│       ├── src/                  # TypeScript API
│       ├── tests/                # JavaScript tests
│       └── bench/                # Benchmarks
├── docs/                         # Documentation
├── .github/workflows/            # CI/CD
└── README.md
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 2. Make Changes

- Write code following the [Code Style](#code-style) guidelines
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run Rust tests
cargo test --package es-module-lexer

# Run JavaScript tests
cd packages/es-module-lexer-rs
pnpm test

# Run full test suite
pnpm test:full
```

### 4. Format and Lint

```bash
# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy -- -D warnings

# Format TypeScript code
cd packages/es-module-lexer-rs
pnpm format

# Lint TypeScript code
pnpm lint
```

### 5. Commit Changes

Follow the [Commit Guidelines](#commit-guidelines) below.

### 6. Push and Create PR

```bash
git push origin your-branch-name
```

Then create a Pull Request on GitHub.

## Testing

### Rust Tests

```bash
# Run all Rust tests
cargo test --package es-module-lexer

# Run specific test
cargo test --package es-module-lexer test_name

# Run tests with output
cargo test --package es-module-lexer -- --nocapture

# Run property-based tests (may take longer)
cargo test --package es-module-lexer --release
```

### JavaScript Tests

```bash
cd packages/es-module-lexer-rs

# Run unit tests
pnpm test

# Run full test suite (includes integration tests)
pnpm test:full

# Run tests in watch mode
pnpm test:watch

# Run specific test file
pnpm vitest run tests/unit.test.ts
```

### Test Coverage

We aim for high test coverage:

- **Unit tests**: Test individual functions and components
- **Integration tests**: Test complete parsing workflows
- **Property-based tests**: Validate correctness across random inputs
- **Comparison tests**: Ensure output matches original es-module-lexer

### Writing Tests

**Rust Unit Test Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_import() {
        let source = r#"import foo from 'bar';"#;
        let mut lexer = Lexer::new(source);
        let result = lexer.parse().unwrap();
        
        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.imports[0].import_type, ImportType::Static);
    }
}
```

**JavaScript Test Example**:
```typescript
import { describe, test, expect } from 'vitest';
import { parse } from '../src';

describe('parse', () => {
  test('should parse static import', () => {
    const source = `import foo from 'bar';`;
    const result = parse(source);
    
    expect(result.imports).toHaveLength(1);
    expect(result.imports[0].t).toBe(1); // Static import
    expect(result.imports[0].n).toBe('bar');
  });
});
```

## Benchmarking

### Running Benchmarks

```bash
# Rust benchmarks (using criterion)
cargo bench --package es-module-lexer

# JavaScript benchmarks
cd packages/es-module-lexer-rs
pnpm bench

# Comparison benchmarks (Rust vs Original)
pnpm bench:comparison

# Real-world code benchmarks
pnpm bench:real-world
```

### Adding Benchmarks

**Rust Benchmark Example**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_simple(c: &mut Criterion) {
    let source = r#"import foo from 'bar';"#;
    
    c.bench_function("parse simple import", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source));
            lexer.parse().unwrap()
        });
    });
}

criterion_group!(benches, bench_parse_simple);
criterion_main!(benches);
```

**JavaScript Benchmark Example**:
```typescript
import { bench, describe } from 'vitest';
import { parse } from '../src';

describe('Performance', () => {
  const source = `import foo from 'bar';`;
  
  bench('parse simple import', () => {
    parse(source);
  });
});
```

## Code Style

### Rust Code Style

We follow standard Rust conventions:

- **Formatting**: Use `cargo fmt` (rustfmt)
- **Linting**: Use `cargo clippy` with no warnings
- **Naming**:
  - `snake_case` for functions and variables
  - `PascalCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants
- **Documentation**: Add doc comments for public APIs
- **Error Handling**: Use `Result` type, avoid panics in library code

**Example**:
```rust
/// Parses JavaScript module source code.
///
/// # Arguments
///
/// * `source` - The source code to parse
///
/// # Returns
///
/// Returns `Ok(ParseResult)` on success, or `Err(LexerError)` on failure.
///
/// # Examples
///
/// ```
/// use es_module_lexer::parse;
///
/// let source = r#"import foo from 'bar';"#;
/// let result = parse(source).unwrap();
/// assert_eq!(result.imports.len(), 1);
/// ```
pub fn parse(source: &str) -> Result<ParseResult, LexerError> {
    let mut lexer = Lexer::new(source);
    lexer.parse()
}
```

### TypeScript Code Style

- **Formatting**: Use `oxfmt` (included in package)
- **Linting**: Use `oxlint` (included in package)
- **Naming**:
  - `camelCase` for functions and variables
  - `PascalCase` for types and interfaces
- **Types**: Always use explicit types, avoid `any`

**Example**:
```typescript
/**
 * Parses JavaScript module source code.
 * 
 * @param source - The source code to parse
 * @returns Parse result with imports and exports
 */
export function parse(source: string): ParseResult {
  return nativeParse(source);
}
```

### Best Practices

1. **Safety First**: Never use `unsafe` without thorough justification and documentation
2. **No Panics**: Library code should not panic; use `Result` for error handling
3. **Zero-Copy**: Prefer slices (`&[u8]`, `&str`) over owned types when possible
4. **Pre-allocate**: Use `Vec::with_capacity` when size is known
5. **Inline Judiciously**: Use `#[inline]` for small, frequently-called functions
6. **Document Public APIs**: All public functions, types, and modules need documentation
7. **Test Edge Cases**: Include tests for empty input, large input, invalid input

## Commit Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/):

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks (dependencies, build, etc.)
- `ci`: CI/CD changes

### Examples

```
feat(parser): add support for import defer syntax

Implements parsing for the new import defer proposal.

Closes #123
```

```
fix(lexer): handle unterminated strings correctly

Previously, unterminated strings would cause a panic.
Now returns LexerError::UnterminatedString.

Fixes #456
```

```
docs: update README with performance benchmarks

Added comparison table showing performance vs original implementation.
```

## Pull Request Process

### Before Submitting

1. ✅ All tests pass (`pnpm test` and `cargo test`)
2. ✅ Code is formatted (`cargo fmt` and `pnpm format`)
3. ✅ No lint warnings (`cargo clippy` and `pnpm lint`)
4. ✅ Documentation is updated
5. ✅ Commit messages follow guidelines
6. ✅ Branch is up to date with main

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
Describe how you tested your changes

## Checklist
- [ ] Tests pass
- [ ] Code is formatted
- [ ] Documentation updated
- [ ] No lint warnings
```

### Review Process

1. Automated checks run (CI/CD)
2. Maintainer reviews code
3. Address feedback if needed
4. Maintainer approves and merges

### After Merge

- Your branch will be deleted
- Changes will be included in the next release
- You'll be credited in the changelog

## Areas for Contribution

### Good First Issues

- Adding test cases
- Improving documentation
- Fixing typos
- Adding examples

### Feature Contributions

- Support for new JavaScript syntax
- Performance optimizations (within safety constraints)
- Better error messages
- Additional benchmarks

### Documentation Contributions

- Tutorial articles
- API documentation improvements
- Architecture explanations
- Translation to other languages

### What We Won't Accept

- PRs that sacrifice memory safety for performance
- PRs that use `unsafe` without strong justification
- PRs that break API compatibility
- PRs without tests
- PRs that don't follow code style

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Open a GitHub Issue
- **Security**: Email zhiqiangfu6@gmail.com
- **Chat**: (Add Discord/Slack link if available)

## Recognition

Contributors will be:
- Listed in the changelog
- Credited in release notes
- Added to the contributors list

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to es-module-lexer-rs! 🦀

