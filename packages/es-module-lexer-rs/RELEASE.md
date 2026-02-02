# Release Process

This document outlines the step-by-step process for releasing a new version of `es-module-lexer-rs`.

## Pre-Release Checklist

- [ ] All tests passing locally (`pnpm test`)
- [ ] All benchmarks running successfully (`pnpm bench`)
- [ ] Code linted and formatted (`pnpm lint && pnpm format`)
- [ ] CI/CD pipeline passing on main branch
- [ ] CHANGELOG.md updated with release notes
- [ ] Documentation updated if needed

## Release Steps

### 1. Determine Version Number

Follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Example: `0.1.0` → `0.2.0` (new features) or `0.1.1` (bug fixes)

### 2. Update Version Numbers

Run the version sync script:

```bash
cd packages/es-module-lexer-rs
pnpm sync-versions 0.2.0
```

This will update:
- `package.json`
- All platform package.json files in `npm/*/package.json`
- Optional dependencies versions

### 3. Update Cargo.toml

Manually update the version in:
```bash
packages/es-module-lexer-rs/native/Cargo.toml
```

Change:
```toml
[package]
version = "0.2.0"
```

### 4. Update CHANGELOG.md

Add a new section for the release:

```markdown
## [0.2.0] - 2025-01-27

### Added
- New feature X
- New feature Y

### Changed
- Improved performance of Z

### Fixed
- Bug fix for issue #123
```

### 5. Commit Changes

```bash
git add .
git commit -m "chore: release v0.2.0"
git push origin main
```

### 6. Create and Push Git Tag

```bash
git tag v0.2.0
git push origin v0.2.0
```

### 7. Monitor GitHub Actions

1. Go to: https://github.com/Sunny-117/es-module-lexer-rs/actions
2. Watch the "Build" workflow
3. Ensure all platform builds succeed
4. Verify tests pass on all platforms

### 8. Verify npm Publication

Once the workflow completes:

1. Check main package: https://www.npmjs.com/package/es-module-lexer-rs
2. Check platform packages:
   - https://www.npmjs.com/package/es-module-lexer-rs-darwin-x64
   - https://www.npmjs.com/package/es-module-lexer-rs-darwin-arm64
   - https://www.npmjs.com/package/es-module-lexer-rs-darwin-universal
   - https://www.npmjs.com/package/es-module-lexer-rs-linux-x64-gnu
   - https://www.npmjs.com/package/es-module-lexer-rs-linux-arm64-gnu
   - https://www.npmjs.com/package/es-module-lexer-rs-win32-x64-msvc
   - https://www.npmjs.com/package/es-module-lexer-rs-freebsd-x64

### 9. Test Installation

Test the published package on different platforms:

```bash
# Create a test directory
mkdir test-install
cd test-install
npm init -y

# Install the package
npm install es-module-lexer-rs

# Test it
node -e "const { parse } = require('es-module-lexer-rs'); console.log(parse('import foo from \"bar\"'))"
```

### 10. Create GitHub Release

1. Go to: https://github.com/Sunny-117/es-module-lexer-rs/releases/new
2. Select the tag: `v0.2.0`
3. Title: `v0.2.0`
4. Description: Copy from CHANGELOG.md
5. Attach artifacts (optional)
6. Click "Publish release"

### 11. Announce Release

- Update README.md if needed
- Post on relevant forums/communities
- Update documentation site if applicable

## Manual Release (Fallback)

If automated release fails, you can publish manually:

### 1. Build All Platforms Locally

This requires access to all target platforms or cross-compilation setup.

```bash
# macOS
pnpm build:rust --target x86_64-apple-darwin
pnpm build:rust --target aarch64-apple-darwin

# Create universal binary
lipo -create \
  es-module-lexer-rs-native.darwin-x64.node \
  es-module-lexer-rs-native.darwin-arm64.node \
  -output es-module-lexer-rs-native.darwin-universal.node

# Linux (requires cross-compilation setup)
pnpm build:rust --target x86_64-unknown-linux-gnu
pnpm build:rust --target aarch64-unknown-linux-gnu

# Windows (requires Windows or cross-compilation)
pnpm build:rust --target x86_64-pc-windows-msvc
```

### 2. Build TypeScript

```bash
pnpm build:ts
```

### 3. Publish Main Package

```bash
npm publish --access public
```

### 4. Publish Platform Packages

```bash
pnpm publish:platforms
```

## Rollback

If a release has critical issues:

### 1. Deprecate the Version

```bash
npm deprecate es-module-lexer-rs@0.2.0 "Critical bug, use 0.1.0 instead"
```

### 2. Publish Hotfix

Follow the release process with a patch version (e.g., `0.2.1`)

### 3. Update Documentation

Clearly communicate the issue and recommended version.

## Troubleshooting

### Build Fails on Specific Platform

- Check GitHub Actions logs for details
- Verify Rust toolchain is up to date
- Check for platform-specific dependencies

### npm Publish Fails

- Verify npm authentication: `npm whoami`
- Check package name availability
- Ensure version doesn't already exist

### Binary Not Loading

- Verify .node file naming matches platform detection in index.js
- Check file permissions
- Test on actual target platform

## Post-Release

- [ ] Monitor npm download stats
- [ ] Watch for bug reports
- [ ] Update project roadmap
- [ ] Plan next release

## Support

For release issues:
- GitHub Issues: https://github.com/Sunny-117/es-module-lexer-rs/issues
- Email: zhiqiangfu6@gmail.com
