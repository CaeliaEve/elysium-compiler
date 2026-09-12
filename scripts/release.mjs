import { createHash } from 'node:crypto';
import { constants, existsSync, mkdirSync, readFileSync, writeFileSync, copyFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

if (process.argv.length > 3) throw new Error('Usage: node scripts/release.mjs [output-directory]');
const root = fileURLToPath(new URL('../', import.meta.url));
const platform = { win32: 'windows', linux: 'linux', darwin: 'macos' }[process.platform];
if (!platform) throw new Error('Unsupported native release platform: ' + process.platform);
const target = `${platform}-${process.arch}`;
const output = resolve(process.argv[2] ?? join(root, 'release', target));
if (existsSync(output)) throw new Error('Release directory already exists: ' + output);
const executable = process.platform === 'win32' ? 'elysium-compiler.exe' : 'elysium-compiler';
const binary = join(root, 'target/release', executable);
execFileSync('cargo', ['build', '--release', '-p', 'elysium-compiler'], { cwd: root, stdio: 'inherit' });
const version = execFileSync(binary, ['--version'], { encoding: 'utf8' }).trim().match(/^elysium-compiler ([0-9A-Za-z.+-]+)$/)?.[1];
if (!version) throw new Error('The release binary did not report its version');
execFileSync(process.execPath, [join(root, 'scripts/check-release.mjs'), binary], { cwd: root, stdio: 'inherit' });

const files = new Map([
  [executable, `target/release/${executable}`],
  ['README.md', 'README.md'],
  ['docs/refactor.md', 'docs/refactor.md'],
  ...['schema.json', 'index.cjs', 'index.d.ts', 'package.json'].map(name => [`contracts/${name}`, `contracts/${name}`]),
]);
mkdirSync(dirname(output), { recursive: true });
mkdirSync(output);
const manifest = [];
for (const [name, source] of files) {
  const file = join(output, name);
  mkdirSync(dirname(file), { recursive: true });
  copyFileSync(join(root, source), file, constants.COPYFILE_EXCL);
  const bytes = readFileSync(file);
  manifest.push({ path: name, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') });
}
writeFileSync(join(output, 'files.json'), JSON.stringify({ target, version, files: manifest }, null, 2) + '\n', { flag: 'wx' });
console.log(JSON.stringify({ path: output, target, version }));
