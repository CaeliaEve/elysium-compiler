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
const targetRoot = resolve(root, process.env.CARGO_TARGET_DIR ?? 'target');
const binary = join(targetRoot, 'release', executable);
execFileSync('cargo', ['build', '--release', '-p', 'elysium-compiler'], { cwd: root, stdio: 'inherit' });
const version = execFileSync(binary, ['--version'], { encoding: 'utf8' }).trim().match(/^elysium-compiler ([0-9A-Za-z.+-]+)$/)?.[1];
if (!version) throw new Error('The release binary did not report its version');
execFileSync(process.execPath, [join(root, 'scripts/check-release.mjs'), binary], { cwd: root, stdio: 'inherit' });

const files = new Map([
  [executable, binary],
  ['README.md', 'README.md'],
  ['docs/refactor.md', 'docs/refactor.md'],
  ['docs/quantities.md', 'docs/quantities.md'],
  ['docs/resources.md', 'docs/resources.md'],
  ['docs/captures.md', 'docs/captures.md'],
  ['docs/native-usage.md', 'docs/native-usage.md'],
  ['docs/native-batch.md', 'docs/native-batch.md'],
  ['docs/targeted-native-returns.md', 'docs/targeted-native-returns.md'],
  ['docs/filter-results.md', 'docs/filter-results.md'],
  ['docs/blast.md', 'docs/blast.md'],
  ['docs/casting.md', 'docs/casting.md'],
  ['docs/metadata.md', 'docs/metadata.md'],
  ['docs/native-diagram-bindings.md', 'docs/native-diagram-bindings.md'],
  ['docs/native-construction-display.md', 'docs/native-construction-display.md'],
  ['docs/qed.md', 'docs/qed.md'],
  ['docs/galaxyspace.md', 'docs/galaxyspace.md'],
  ['docs/floating-flowers.md', 'docs/floating-flowers.md'],
  ['docs/genetics.md', 'docs/genetics.md'],
  ['docs/ore-groups.md', 'docs/ore-groups.md'],
  ...['schema.json', 'index.cjs', 'index.d.ts', 'package.json','metadata.json','validators.cjs','numeric-formats.cjs'].map(name => [`contracts/${name}`, `contracts/${name}`]),
]);
mkdirSync(dirname(output), { recursive: true });
mkdirSync(output);
const manifest = [];
for (const [name, source] of files) {
  const file = join(output, name);
  mkdirSync(dirname(file), { recursive: true });
  copyFileSync(resolve(root, source), file, constants.COPYFILE_EXCL);
  const bytes = readFileSync(file);
  manifest.push({ path: name, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') });
}
writeFileSync(join(output, 'files.json'), JSON.stringify({ target, version, files: manifest }, null, 2) + '\n', { flag: 'wx' });
console.log(JSON.stringify({ path: output, target, version }));
