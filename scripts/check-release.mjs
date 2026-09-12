import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
import { existsSync, readFileSync, mkdtempSync, realpathSync, rmSync, statSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { resolve, join, dirname, basename } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const binary = process.argv[2] ? resolve(process.argv[2]) : null;
if (!binary || !existsSync(binary) || !statSync(binary).isFile() || process.argv.length > 4) {
  console.error('Usage: node scripts/check-release.mjs <compiler-binary> [expected-sha256]');
  process.exit(1);
}
const expected = process.argv[3] ?? null;
const sha256 = createHash('sha256').update(readFileSync(binary)).digest('hex');
if (expected && sha256 !== expected) {
  console.error(`checksum mismatch: expected ${expected}, got ${sha256}`);
  process.exit(1);
}
const root = fileURLToPath(new URL('../', import.meta.url));
const temporaryRoot = realpathSync(tmpdir());
const temporary = mkdtempSync(join(temporaryRoot, 'elysium-release-'));
function run(args) {
  const result = spawnSync(binary, args, { encoding: 'utf8', shell: false, timeout: 60000, maxBuffer: 4 * 1024 * 1024 });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr.trim() || `Compiler exited with ${result.status}`);
  return JSON.parse(result.stdout);
}
try {
  const schema = join(temporary, 'schema.json');
  run(['schema', '--output', schema]);
  assert.deepEqual(JSON.parse(readFileSync(schema, 'utf8')), JSON.parse(readFileSync(join(root, 'contracts/schema.json'), 'utf8')), 'Release schema differs from the packaged contract');
  const input = join(root, 'contracts/fixtures/source'), output = join(temporary, 'catalog');
  run(['inspect', '--input', input]);
  const publication = run(['compile', '--input', input, '--output', output]);
  const repeated = run(['compile', '--input', input, '--output', output]);
  assert.equal(publication.id, repeated.id, 'Release compilation is not deterministic');
  assert.equal(publication.source, JSON.parse(readFileSync(join(input, 'manifest.json'), 'utf8')).id);
  for (const kind of ['items', 'recipes', 'textures', 'materials', 'circuits', 'species', 'mutations', 'lineage', 'structures', 'blocks', 'builds', 'models', 'shapes', 'aspects', 'research']) assert.ok(publication.counts[kind] > 0, `No fixture ${kind} in release output`);
  run(['check', '--input', output]);
} finally {
  // Delete only the temporary directory created by this invocation, never a caller-supplied path.
  if (dirname(realpathSync(temporary)) !== temporaryRoot || !basename(temporary).startsWith('elysium-release-')) throw new Error('Temporary cleanup path changed');
  rmSync(temporary, { recursive: true });
}
console.log(JSON.stringify({ binary, sha256, status: 'ok' }, null, 2));
