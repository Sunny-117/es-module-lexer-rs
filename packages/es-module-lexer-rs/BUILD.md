# Build and Publish Guide

This document describes how to build and publish `es-module-lexer-rs` for multiple platforms.

## Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Node.js**: Version 18 or higher
- **pnpm**: Version 8 or higher
- **napi-rs CLI**: Installed via `@napi-rs/cli` (included in devDependencies)

## Local Development

### Building for Your Platform

```bash
# Install dependencies
pnpm install

# Build Rust native module
cd packages/es-module-lexer-rs
pnpm build:rust

# Build TypeScript
pnpm build:ts

# Or build everything
pnpm build
```

### Testing

```bash
# Run tests
pnpm test

# Run full test suite
pnpm test:full

# Run benchmarks
pnpm bench
```

## Cross-Platform Building

### Building for Specific Platforms

```bash
cd packages/es-module-lexer-rs

# macOS x64
pnpm build:rust --target x86_64-apple-darwin

# macOS ARM64
pnpm build:rust --target aarch64-apple-darwin

# Linux x64
pnpm build:rust --target x86_64-unknown-linux-gnu

# Linux ARM64
pnpm build:rust --target aarch64-unknown-linux-gnu

# Windows x64
pnpm build:rust --target x86_64-pc-windows-msvc
```

### Adding Rust Targets

If you need to build for a platform that isn't installed:

```bash
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-pc-windows-msvc
```

### Cross-Compilation Setup

#### Linux ARM64 on Linux x64

```bash
sudo apt-get update
sudo apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
```

#### macOS Universal Binary

```bash
# Build both architectures
pnpm build:rust --target x86_64-apple-darwin
pnpm build:rust --target aarch64-apple-darwin

# Create universal binary
lipo -create \
  es-module-lexer-rs-native.darwin-x64.node \
  es-module-lexer-rs-native.darwin-arm64.node \
  -output es-module-lexer-rs-native.darwin-universal.node
```

## CI/CD with GitHub Actions

The project uses GitHub Actions for automated multi-platform builds. The workflow is defined in `.github/workflows/build.yml`.

### Workflow Triggers

- **Push to main**: Builds all platforms
- **Pull requests**: Builds all platforms
- **Tags (v*)**: Builds all platforms and publishes to npm
- **Manual dispatch**: Can be triggered manually

### Build Matrix

The workflow builds for:
- Linux x64 (glibc)
- Linux ARM64 (glibc)
- macOS x64
- macOS ARM64
- macOS Universal
- Windows x64
- FreeBSD x64

### Artifacts

After each build, the workflow uploads artifacts that can be downloaded for testing or manual publishing.

## Publishing

### Automated Publishing (Recommended)

1. Update version in `package.json` and `Cargo.toml`
2. Commit changes
3. Create and push a git tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
4. GitHub Actions will automatically:
   - Build all platforms
   - Run tests
   - Publish to npm (requires `NPM_TOKEN` secret)

### Manual Publishing

#### Prerequisites

- All platform binaries built and available
- npm authentication configured (`npm login`)

#### Steps

1. **Build all platforms** (or download from CI artifacts)

2. **Copy binaries to package directory**:
   ```bash
   cd packages/es-module-lexer-rs
   # Ensure all .node files are in the package root
   ```

3. **Build TypeScript**:
   ```bash
   pnpm build:ts
   ```

4. **Publish main package**:
   ```bash
   npm publish --access public
   ```

5. **Publish platform packages**:
   ```bash
   pnpm publish:platforms
   ```

## Platform Package Structure

Each platform has its own npm package:

- `es-module-lexer-rs-darwin-x64`
- `es-module-lexer-rs-darwin-arm64`
- `es-module-lexer-rs-darwin-universal`
- `es-module-lexer-rs-linux-x64-gnu`
- `es-module-lexer-rs-linux-arm64-gnu`
- `es-module-lexer-rs-win32-x64-msvc`
- `es-module-lexer-rs-freebsd-x64`

These are listed as `optionalDependencies` in the main package. When users install `es-module-lexer-rs`, npm will automatically install the appropriate platform package.

## Binary Loading

The `index.js` file automatically detects the user's platform and loads the correct native binary:

1. First tries to load from the main package (local .node file)
2. Falls back to the platform-specific package
3. Throws an error if no compatible binary is found

## Troubleshooting

### Build Failures

**Error: linker not found**
- Install the appropriate cross-compilation toolchain
- For Linux ARM64: `sudo apt-get install gcc-aarch64-linux-gnu`

**Error: target not installed**
- Add the target: `rustup target add <target-triple>`

**Error: napi build failed**
- Ensure `@napi-rs/cli` is installed: `pnpm install`
- Check Rust version: `rustc --version` (should be 1.70+)

### Publishing Failures

**Error: 401 Unauthorized**
- Run `npm login` to authenticate
- Ensure you have publish permissions for the package

**Error: version already exists**
- Update the version number in `package.json`
- Use `npm version patch/minor/major` to bump version

**Error: platform package not found**
- Ensure all .node files are built before publishing
- Check that npm/platform directories exist

## Version Management

When releasing a new version:

1. Update version in all relevant files:
   - `packages/es-module-lexer-rs/package.json`
   - `packages/es-module-lexer-rs/native/Cargo.toml`
   - All platform package.json files in `npm/*/package.json`

2. Update CHANGELOG.md with release notes

3. Create git tag and push

## Security

- Never commit npm tokens or credentials
- Use GitHub Secrets for CI/CD authentication
- Review dependencies regularly for vulnerabilities
- Keep Rust toolchain updated

## Support

For issues or questions:
- GitHub Issues: https://github.com/Sunny-117/es-module-lexer-rs/issues
- Email: zhiqiangfu6@gmail.com
