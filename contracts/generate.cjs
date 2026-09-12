const { readFileSync, writeFileSync } = require('node:fs');
const { compile } = require('json-schema-to-typescript');

async function main() {
  const schema = JSON.parse(readFileSync(`${__dirname}/schema.json`, 'utf8'));
  const types = await compile(schema, 'Contract', {
    bannerComment: '/* Generated from the Rust contract. Run cargo schema and npm run generate to update. */',
    additionalProperties: false,
    unreachableDefinitions: true,
  });
  writeFileSync(`${__dirname}/index.d.ts`, `${types}\n
export declare function assertManifest(value: unknown): asserts value is Manifest;
export declare const formats: { readonly source: { readonly name: string; readonly revision: number }; readonly catalog: { readonly name: string; readonly revision: number } };
export declare const collections: readonly Table['kind'][];
export declare const topicKinds: readonly TopicKind[];
export declare function assertPointer(value: unknown): asserts value is Pointer;
export declare function assertTable(value: unknown): asserts value is Table;
export declare function decodeTable(bytes: Uint8Array, kind?: Table['kind']): Table;
export declare function canonical(value: unknown): string;
`);
}
main().catch(error => { console.error(error); process.exitCode = 1; });
