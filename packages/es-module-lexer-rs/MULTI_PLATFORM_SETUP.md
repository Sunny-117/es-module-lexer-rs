# Multi-Platform Build Setup - Summary

This document summarizes the multi-platform build and publishing infrastructure that has been configured for `es-module-lexer-rs`.

## What Was Implemented

### 1. GitHub Actions Workflow (`.github/workflows/build.yml`)

A comprehensive CI/CD pipeline that:

- **Builds native binaries** for all supported platforms:
  - Linux x64 (glibc)
  - Linux ARM64 (glibc)
  - macOS x64 (Intel)
  - macOS ARM64 (Apple Silicon)
  - macOS Universal (x64 + ARM64 combined)
  - Windows x64 (MSVC)
  - FreeBSD x64

- **Tests bindings** on each platform to ensure they work correctly

- **Creates universal macOS binary** by combining x64 and ARM64 binaries using `lipo`

- **Automatically publishes to npm** when a version tag (e.g., `v0.1.0`) is pushed

### 2. Platform-Specific npm Packages

Created package.json templates for each platform in `npm/*/package.json`:

- `es-module-lexer-rs-darwin-x64`
- `es-module-lexer-rs-darwin-arm64`
- `es-module-lexer-rs-darwin-universal`
- `es-module-lexer-rs-linux-x64-gnu`
- `es-module-lexer-rs-linux-arm64-gnu`
- `es-module-lexer-rs-win32-x64-msvc`
- `es-module-lexer-rs-freebsd-x64`

These packages are listed as `optionalDependencies` in the main package, allowing npm to automatically install the correct one for each platform.

### 3. Automated Publishing Scripts

**`scripts/publish-platform-packages.sh`**
- Publishes all platform-specific packages to npm
- Copies the correct .node file to each platform directory
- Handles errors gracefully

**`scripts/sync-versions.js`**
- Synchronizes version numbers across all package.json files
- Updates optionalDependencies versions
- Ensures consistency across the monorepo

### 4. Package Configuration Updates

**Main `package.json` updates:**
- Added `artifacts`, `prepublishOnly`, and `version` scripts
- Added `sync-versions` script for version management
- Added `optionalDependencies` for platform packages
- Added repository, bugs, and homepage fields
- Updated `files` array to include README and LICENSE

**`.npmignore` file:**
- Excludes source files, tests, and build artifacts from npm package
- Keeps package size minimal

### 5. Documentation

**`BUILD.md`**
- Comprehensive guide for building on different platforms
- Cross-compilation instructions
- CI/CD workflow explanation
- Troubleshooting section

**`RELEASE.md`**
- Step-by-step release process
- Pre-release checklist
- Manual release fallback instructions
- Rollback procedures

**`npm/README.md`**
- Explains platform-specific packages
- Documents naming conventions
- Describes how the system works

## How It Works

### Installation Flow

1. User runs: `npm install es-module-lexer-rs`
2. npm installs the main package
3. npm tries to install the appropriate platform package from `optionalDependencies`
4. The main package's `index.js` detects the platform and loads the correct binary:
   - First tries local .node file (from main package)
   - Falls back to platform-specific package
   - Throws error if no compatible binary found

### Build Flow

1. Developer pushes a git tag (e.g., `v0.1.0`)
2. GitHub Actions triggers the build workflow
3. Workflow builds binaries for all platforms in parallel
4. Each build uploads its artifact
5. Test jobs download artifacts and run tests
6. Universal macOS binary is created from x64 and ARM64 binaries
7. If all tests pass, packages are published to npm

### Release Flow

1. Update version numbers using `pnpm sync-versions <version>`
2. Update `native/Cargo.toml` version manually
3. Update `CHANGELOG.md`
4. Commit changes: `git commit -m "chore: release v0.1.0"`
5. Create tag: `git tag v0.1.0`
6. Push tag: `git push origin v0.1.0`
7. GitHub Actions handles the rest automatically

## Key Features

### ✅ Automated Multi-Platform Builds
No need to manually build on each platform - GitHub Actions handles it all.

### ✅ Automatic Platform Detection
Users don't need to specify their platform - it's detected automatically.

### ✅ Fallback Mechanism
If the local binary isn't found, falls back to platform-specific package.

### ✅ Universal macOS Binary
Single binary works on both Intel and Apple Silicon Macs.

### ✅ Version Synchronization
Script ensures all packages have consistent version numbers.

### ✅ Comprehensive Testing
Binaries are tested on actual target platforms before publishing.

### ✅ Minimal Package Size
Only necessary files are included in npm packages.

## Requirements Met

This implementation satisfies the following requirements from the design document:

- **需求 14.1**: Build system for Linux (x64, arm64) ✅
- **需求 14.2**: Build system for macOS (x64, arm64) ✅
- **需求 14.3**: Build system for Windows (x64) ✅
- **需求 14.6**: Package includes all platform binaries ✅
- **需求 14.7**: Automatic platform selection on install ✅

## Next Steps

To use this setup:

1. **Configure npm token** in GitHub repository secrets:
   - Go to repository Settings → Secrets → Actions
   - Add secret named `NPM_TOKEN` with your npm access token

2. **Test the workflow**:
   ```bash
   # Make a test release
   pnpm sync-versions 0.1.0
   git add .
   git commit -m "chore: release v0.1.0"
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. **Monitor the build**:
   - Watch GitHub Actions for build progress
   - Check npm for published packages

4. **Test installation**:
   ```bash
   npm install es-module-lexer-rs
   ```

## Maintenance

### Adding New Platforms

To add support for a new platform:

1. Add build configuration to `.github/workflows/build.yml`
2. Create `npm/<platform>/package.json`
3. Add platform to `optionalDependencies` in main package.json
4. Update `index.js` platform detection logic (if needed)
5. Update documentation

### Updating Dependencies

- Keep `@napi-rs/cli` updated for latest napi-rs features
- Update Rust toolchain regularly for security and performance
- Monitor GitHub Actions for deprecated features

## Support

For issues or questions about the build system:
- Check `BUILD.md` for detailed build instructions
- Check `RELEASE.md` for release procedures
- Open an issue on GitHub
- Contact: zhiqiangfu6@gmail.com

---

**Implementation Date**: 2025-01-27
**Status**: ✅ Complete
