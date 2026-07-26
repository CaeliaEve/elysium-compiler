import crypto from 'node:crypto';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const compiler = resolve(process.argv[2] ?? join(repoRoot, 'target', 'debug', 'elysium-compiler.exe'));
const fixture = join(repoRoot, 'crates', 'elysium-compiler-core', 'fixtures', 'raw-export-texture-atlas');
const catalogPath = join(
  repoRoot,
  'crates',
  'elysium-compiler-core',
  'fixtures',
  'golden',
  'pre-session-raw-export-texture-atlas.sha256.json',
);
const output = mkdtempSync(join(tmpdir(), 'elysium-golden-output-'));
const receipt = mkdtempSync(join(tmpdir(), 'elysium-golden-receipt-'));

function sha256(path) {
  return crypto.createHash('sha256').update(readFileSync(path)).digest('hex');
}

try {
  const result = spawnSync(compiler, [
    'compile',
    '--input', fixture,
    '--output', output,
    '--report', join(receipt, 'compiler-report.json'),
    '--scope', 'all',
    '--threads', '1',
    '--strict',
  ], { cwd: repoRoot, encoding: 'utf8' });
  if (result.status !== 0) {
    throw new Error(`golden compiler failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`);
  }
  const pointer = JSON.parse(readFileSync(join(output, 'current.json'), 'utf8'));
  const generationRoot = join(output, ...pointer.relativePath.split('/'));
  const previous = JSON.parse(readFileSync(catalogPath, 'utf8'));
  const artifacts = Object.fromEntries(Object.keys(previous.artifacts).map((relativePath) => [
    relativePath,
    sha256(join(generationRoot, ...relativePath.split('/'))),
  ]));
  const catalog = {
    schemaVersion: 'elysium-compiler/deterministic-artifact-hashes/v1',
    fixture: 'raw-export-texture-atlas',
    artifacts,
  };
  writeFileSync(catalogPath, `${JSON.stringify(catalog, null, 2)}\n`, 'utf8');
  console.log(JSON.stringify({ status: 'updated', compiler, fixture, artifactCount: Object.keys(artifacts).length, catalogPath }, null, 2));
} finally {
  rmSync(output, { recursive: true, force: true });
  rmSync(receipt, { recursive: true, force: true });
}
