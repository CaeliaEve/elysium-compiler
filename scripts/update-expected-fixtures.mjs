import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const compiler = resolve(process.argv[2] ?? join(repoRoot, 'target', 'debug', 'elysium-compiler.exe'));
const fixturesRoot = join(repoRoot, 'crates', 'elysium-compiler-core', 'fixtures');
const commonUiReports = [
  'recipes/ui-payload-index.json',
  'rust/runtime-manifest.json',
  'rust/native-ui-layout-report.json',
  'rust/ui-pack/ui_pack_report.json',
  'rust/ui-pack/ui_assets.manifest.json',
  'rust/ui-pack/ui_template_catalog.json',
  'rust/ui-pack/ui_template_binding_index.json',
  'rust/ui-pack/ui_family_census.json',
  'rust/integrity.json',
  'rust/raw-export-abi-validation-report.json',
  'rust/native-ui-export-abi-validation-report.json',
  'rust/ui-pack-abi-validation-report.json',
  'rust/pack-validation-report.json',
];
const fixtures = [
  { name: 'raw-export-native-ui-gt', scope: 'native-ui', paths: commonUiReports },
  { name: 'raw-export-semantic-background-only', scope: 'native-ui', paths: commonUiReports },
  {
    name: 'raw-export-sharded-recipes',
    scope: 'native-ui',
    paths: [
      'recipes/ui-payload-index.json',
      'rust/ui-pack/ui_template_catalog.json',
      'rust/ui-pack/ui_template_binding_index.json',
      'rust/ui-pack/ui_family_census.json',
      'rust/raw-export-abi-validation-report.json',
      'rust/native-ui-export-abi-validation-report.json',
      'rust/ui-pack-abi-validation-report.json',
      'rust/pack-validation-report.json',
    ],
  },
  {
    name: 'raw-export-texture-atlas',
    scope: 'all',
    paths: [
      'recipes/ui-payload-index.json',
      'rust/runtime-manifest.json',
      'rust/missing-texture-report.json',
      'rust/suspicious-texture-report.json',
      'rust/ui-pack/ui_template_catalog.json',
      'rust/ui-pack/ui_template_binding_index.json',
      'rust/ui-pack/ui_family_census.json',
      'rust/raw-export-abi-validation-report.json',
      'rust/native-ui-export-abi-validation-report.json',
      'rust/ui-pack-abi-validation-report.json',
    ],
  },
];

for (const fixture of fixtures) {
  const output = mkdtempSync(join(tmpdir(), `elysium-${fixture.name}-`));
  const receipt = mkdtempSync(join(tmpdir(), `elysium-${fixture.name}-receipt-`));
  try {
    const result = spawnSync(compiler, [
      'compile',
      '--input', join(fixturesRoot, fixture.name),
      '--output', output,
      '--report', join(receipt, 'compiler-report.json'),
      '--scope', fixture.scope,
      '--threads', '1',
      '--strict',
    ], { cwd: repoRoot, encoding: 'utf8' });
    if (result.status !== 0) {
      throw new Error(`${fixture.name} compile failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`);
    }
    const pointer = JSON.parse(readFileSync(join(output, 'current.json'), 'utf8'));
    const generationRoot = join(output, ...pointer.relativePath.split('/'));
    const expectedRoot = join(fixturesRoot, 'expected', fixture.name);
    for (const relativePath of fixture.paths) {
      const target = join(expectedRoot, ...relativePath.split('/'));
      mkdirSync(dirname(target), { recursive: true });
      copyFileSync(join(generationRoot, ...relativePath.split('/')), target);
    }
    console.log(`${fixture.name}: updated ${fixture.paths.length} expected artifacts`);
  } finally {
    rmSync(output, { recursive: true, force: true });
    rmSync(receipt, { recursive: true, force: true });
  }
}
