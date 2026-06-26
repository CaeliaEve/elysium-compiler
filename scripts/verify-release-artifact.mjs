import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';

const binary = resolve(process.argv[2] ?? '');
if (!binary || !existsSync(binary)) {
  console.error('Usage: node scripts/verify-release-artifact.mjs <elysium-compiler-binary> [expected-sha256]');
  process.exit(1);
}
const expected = process.argv[3] ?? null;
const sha256 = createHash('sha256').update(readFileSync(binary)).digest('hex');
if (expected && sha256 !== expected) {
  console.error(`checksum mismatch: expected ${expected}, got ${sha256}`);
  process.exit(1);
}
for (const args of [['--help'], ['schemas']]) {
  const result = spawnSync(binary, args, { stdio: 'inherit', shell: false });
  if ((result.status ?? 1) !== 0) process.exit(result.status ?? 1);
}
console.log(JSON.stringify({ binary, sha256, status: 'ok' }, null, 2));
