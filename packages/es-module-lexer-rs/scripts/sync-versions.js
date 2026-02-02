#!/usr/bin/env node

/**
 * Script to synchronize version numbers across all package.json files
 * Usage: node scripts/sync-versions.js <new-version>
 */

const fs = require('fs');
const path = require('path');

const newVersion = process.argv[2];

if (!newVersion) {
  console.error('Usage: node scripts/sync-versions.js <new-version>');
  console.error('Example: node scripts/sync-versions.js 0.2.0');
  process.exit(1);
}

// Validate version format (basic semver check)
if (!/^\d+\.\d+\.\d+(-[a-zA-Z0-9.-]+)?$/.test(newVersion)) {
  console.error('Error: Invalid version format. Expected semver format (e.g., 1.0.0 or 1.0.0-beta.1)');
  process.exit(1);
}

const packageDir = path.join(__dirname, '..');
const npmDir = path.join(packageDir, 'npm');

// Files to update
const filesToUpdate = [
  path.join(packageDir, 'package.json'),
  path.join(npmDir, 'darwin-x64', 'package.json'),
  path.join(npmDir, 'darwin-arm64', 'package.json'),
  path.join(npmDir, 'darwin-universal', 'package.json'),
  path.join(npmDir, 'linux-x64-gnu', 'package.json'),
  path.join(npmDir, 'linux-arm64-gnu', 'package.json'),
  path.join(npmDir, 'win32-x64-msvc', 'package.json'),
  path.join(npmDir, 'freebsd-x64', 'package.json'),
];

console.log(`Updating version to ${newVersion}...`);

let updatedCount = 0;
let errorCount = 0;

for (const file of filesToUpdate) {
  try {
    if (!fs.existsSync(file)) {
      console.warn(`Warning: File not found: ${file}`);
      continue;
    }

    const content = fs.readFileSync(file, 'utf8');
    const pkg = JSON.parse(content);
    const oldVersion = pkg.version;

    pkg.version = newVersion;

    // Also update optionalDependencies versions in main package.json
    if (pkg.optionalDependencies) {
      for (const dep in pkg.optionalDependencies) {
        if (dep.startsWith('es-module-lexer-rs-')) {
          pkg.optionalDependencies[dep] = newVersion;
        }
      }
    }

    fs.writeFileSync(file, JSON.stringify(pkg, null, 2) + '\n', 'utf8');
    console.log(`✓ Updated ${path.relative(packageDir, file)}: ${oldVersion} → ${newVersion}`);
    updatedCount++;
  } catch (error) {
    console.error(`✗ Error updating ${file}:`, error.message);
    errorCount++;
  }
}

console.log(`\nSummary: ${updatedCount} files updated, ${errorCount} errors`);

if (errorCount > 0) {
  process.exit(1);
}

console.log('\nDon\'t forget to also update:');
console.log('  - native/Cargo.toml');
console.log('  - CHANGELOG.md');
console.log('  - Git tag: git tag v' + newVersion);
