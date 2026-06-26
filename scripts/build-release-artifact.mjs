import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync, copyFileSync, rmSync } from 'node:fs';
import { basename, join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const repoRoot = resolve(new URL('..', import.meta.url).pathname.replace(/^\/(.:\/)/, '$1'));
const target = process.argv.includes('--target')
  ? process.argv[process.argv.indexOf('--target') + 1]
  : process.platform === 'win32' ? 'windows-x64' : 'linux-x64';
const outDir = resolve(process.argv.includes('--out-dir') ? process.argv[process.argv.indexOf('--out-dir') + 1] : join(repoRoot, 'dist'));
const exe = process.platform === 'win32' ? 'elysium-compiler.exe' : 'elysium-compiler';
const bin = join(repoRoot, 'target', 'release', exe);

function run(command, args) {
  const result = spawnSync(command, args, { cwd: repoRoot, stdio: 'inherit', shell: false });
  if ((result.status ?? 1) !== 0) process.exit(result.status ?? 1);
}

run('cargo', ['build', '--release', '-p', 'elysium-compiler']);
if (!existsSync(bin)) throw new Error(`release binary missing: ${bin}`);
mkdirSync(outDir, { recursive: true });
const artifactName = `elysium-compiler-0.1.0-${target}-${exe}`;
const artifact = join(outDir, artifactName);
copyFileSync(bin, artifact);
const checksum = createHash('sha256').update(readFileSync(artifact)).digest('hex');
writeFileSync(join(outDir, `${artifactName}.sha256`), `${checksum}  ${basename(artifact)}\n`, 'utf8');
console.log(JSON.stringify({ artifact, sha256: checksum }, null, 2));
