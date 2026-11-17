const { readFileSync, writeFileSync } = require('node:fs');
const { compile } = require('json-schema-to-typescript');
const {createHash}=require('node:crypto');
const Ajv=require('ajv');
const standaloneCode=require('ajv/dist/standalone');
const {_}=require('ajv/dist/compile/codegen');
const numericFormats=require('./numeric-formats.cjs');

function validators(original,directory=__dirname){
  const schema=structuredClone(original);
  // This is the same strict tagged-union optimization used by the previous runtime.
  for(const definition of Object.values(schema.definitions)){
    const variants=definition.oneOf;if(!Array.isArray(variants)||!variants.length)continue;
    const tags=variants.map(variant=>variant.properties?.kind?.enum);
    if(!variants.every((variant,index)=>variant.type==='object'&&variant.required?.includes('kind')
      &&Array.isArray(tags[index])&&tags[index].length===1&&typeof tags[index][0]==='string')
      ||new Set(tags.map(tag=>tag?.[0])).size!==variants.length)continue;
    definition.type='object';definition.discriminator={propertyName:'kind'};
  }
  const ajv=new Ajv({strict:true,allowUnionTypes:true,discriminator:true,
    code:{source:true,formats:_`require('./numeric-formats.cjs')`}});
  for(const [name,format]of Object.entries(numericFormats))ajv.addFormat(name,format);
  ajv.addSchema(schema,'elysium');
  const refs={Manifest:'elysium#/definitions/Manifest',Pointer:'elysium#/definitions/Pointer'};
  for(const [index,table]of schema.definitions.Table.oneOf.entries())refs['table_'+table.properties.kind.enum[0]]='elysium#/definitions/Table/oneOf/'+index;
  const schemaSha256=createHash('sha256').update(readFileSync(`${directory}/schema.json`)).digest('hex');
  const code=standaloneCode(ajv,refs);
  writeFileSync(`${directory}/validators.cjs`,'// Generated from the Rust schema; do not edit.\n'+code+'\nexports.schemaSha256='+JSON.stringify(schemaSha256)+';\n');
  const metadata={schemaSha256,formats:Object.fromEntries([['source','SourceManifest'],['catalog','Manifest']].map(([name,type])=>{
    const fields=original.definitions[type].properties;return [name,{name:fields.format.enum[0],revision:fields.revision.enum[0]}];
  })),collections:original.definitions.Table.oneOf.map(table=>table.properties.kind.enum[0]).sort(),topicKinds:original.definitions.TopicKind.enum};
  writeFileSync(`${directory}/metadata.json`,JSON.stringify(metadata,null,2)+'\n');
}

async function main() {
  const schema = JSON.parse(readFileSync(`${__dirname}/schema.json`, 'utf8'));
  validators(schema);
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
module.exports={generateValidators:validators};
if(require.main===module)main().catch(error => { console.error(error); process.exitCode = 1; });
