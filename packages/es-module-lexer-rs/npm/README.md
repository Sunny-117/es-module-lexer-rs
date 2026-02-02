# Platform-Specific Packages

This directory contains package.json templates for platform-specific npm packages.

## Supported Platforms

- **darwin-x64**: macOS x64 (Intel)
- **darwin-arm64**: macOS ARM64 (Apple Silicon)
- **darwin-universal**: macOS Universal Binary (x64 + ARM64)
- **linux-x64-gnu**: Linux x64 with glibc
- **linux-arm64-gnu**: Linux ARM64 with glibc
- **win32-x64-msvc**: Windows x64 with MSVC
- **freebsd-x64**: FreeBSD x64

## How It Works

When users install `es-module-lexer-rs`, npm will automatically try to install the appropriate platform-specific package as an optional dependency. The main package's `index.js` will then load the correct native binary based on the user's platform.

## Publishing

Platform-specific packages are published automatically by the GitHub Actions workflow when a new version is tagged. They can also be published manually using the `publish-platform-packages.sh` script.

## Package Naming Convention

Platform packages follow the naming convention:
```
es-module-lexer-rs-{platform}-{arch}-{abi}
```

For example:
- `es-module-lexer-rs-darwin-arm64`
- `es-module-lexer-rs-linux-x64-gnu`
- `es-module-lexer-rs-win32-x64-msvc`
