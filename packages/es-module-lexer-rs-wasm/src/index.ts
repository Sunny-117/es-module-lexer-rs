import init, { parse as wasmParse } from '../wasm/index.js';
import { readFile } from 'fs/promises';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

let initialized = false;

async function ensureInitialized() {
  if (!initialized) {
    // Load WASM file for Node.js environment
    const __filename = fileURLToPath(import.meta.url);
    const __dirname = dirname(__filename);
    const wasmPath = join(__dirname, '../wasm/index_bg.wasm');
    const wasmBuffer = await readFile(wasmPath);
    await init(wasmBuffer);
    initialized = true;
  }
}

export interface ImportSpecifier {
  n?: string;
  t: number;
  s: number;
  e: number;
  ss: number;
  se: number;
  d: number;
  a: number;
}

export interface ExportSpecifier {
  n: string;
  ln?: string;
  s: number;
  e: number;
  ls: number;
  le: number;
}

export interface ParseResult {
  imports: ImportSpecifier[];
  exports: ExportSpecifier[];
  facade: boolean;
  hasModuleSyntax: boolean;
}

export async function parse(source: string): Promise<ParseResult> {
  await ensureInitialized();
  return wasmParse(source) as ParseResult;
}

// Synchronous version (requires manual initialization)
export function parseSync(source: string): ParseResult {
  if (!initialized) {
    throw new Error('WASM not initialized. Call parse() first or use init() manually.');
  }
  return wasmParse(source) as ParseResult;
}

// Manual initialization
export { init };
