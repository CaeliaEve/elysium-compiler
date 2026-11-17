// Derive a faster reader from an accepted package without changing its schema or decoder laws.
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const {generateValidators}=require('./generate.cjs');
const [input,output]=process.argv.slice(2);
if(!input||!output||process.argv.length!==4||fs.existsSync(output))throw Error('Supply an accepted package and a NEW output directory');
const hash=data=>crypto.createHash('sha256').update(data).digest('hex');
const original=JSON.parse(fs.readFileSync(path.join(input,'package.json'),'utf8'));
const schemaBytes=fs.readFileSync(path.join(input,'schema.json')),schema=JSON.parse(schemaBytes);
const revision=schema.definitions.Manifest.properties.revision.enum[0];
if(![36,38,39].includes(revision))throw Error('Only explicitly accepted legacy revisions can use this derivation');
const originalRuntime=fs.readFileSync(path.join(input,'index.cjs'),'utf8');
const marker='exports.assertManifest';
if(!originalRuntime.startsWith("const Ajv = require('ajv');")||!originalRuntime.includes(marker))throw Error('Unknown accepted reader implementation');
fs.mkdirSync(output);
for(const file of ['schema.json','index.d.ts'])fs.copyFileSync(path.join(input,file),path.join(output,file),fs.constants.COPYFILE_EXCL);
fs.copyFileSync(path.join(__dirname,'numeric-formats.cjs'),path.join(output,'numeric-formats.cjs'),fs.constants.COPYFILE_EXCL);
generateValidators(schema,output);
const runtimeHead=`const { decode } = require('@msgpack/msgpack');
const metadata=require('./metadata.json'),validators=require('./validators.cjs');
if(metadata.schemaSha256!==validators.schemaSha256)throw Error('Legacy validator/schema metadata mismatch');
exports.formats=Object.freeze(Object.fromEntries(Object.entries(metadata.formats).map(([name,value])=>[name,Object.freeze(value)])));
exports.collections=Object.freeze(metadata.collections);exports.topicKinds=Object.freeze(metadata.topicKinds);
const tableKinds=new Set(metadata.collections);
function validationError(name,errors){
 const details=[...new Set((errors??[]).map(error=>(error.instancePath||'/')+' '+error.message).map(message=>message.length>256?message.slice(0,253)+'...':message))];
 const kept=details.slice(0,6),remaining=details.length-kept.length;
 return Error('Invalid '+name.toLowerCase()+': '+kept.join('; ')+(remaining?'; '+remaining+' additional validation details omitted':''));
}
function assert(name,value){if(name==='Table')return assertTable(value);const validate=validators[name];if(!validate(value))throw validationError(name,validate.errors);}
function assertTable(value){const kind=value&&typeof value==='object'&&!Array.isArray(value)?value.kind:undefined;
 if(!tableKinds.has(kind))throw Error('Invalid table: unknown or missing kind');const validate=validators['table_'+kind];if(!validate(value))throw validationError('Table',validate.errors);}
`;
// Keep the original decoding limits, canonical identities and legacy GUI interpretation.
const suffix=originalRuntime.slice(originalRuntime.indexOf(marker));
if(/\bajv\./.test(suffix)||/\bschema\./.test(suffix))throw Error('Legacy decoder contains an unreviewed compiler dependency');
fs.writeFileSync(path.join(output,'index.cjs'),runtimeHead+suffix,{flag:'wx'});
const version=original.version+(original.version.includes('-')?'.aot.1':'-aot.1');
const pkg={...original,version,description:`AOT compatibility reader for accepted Catalog ${revision}; original schema retained`,
 files:[...new Set([...original.files,'metadata.json','validators.cjs','numeric-formats.cjs'])],dependencies:{...original.dependencies,ajv:require('ajv/package.json').version}};
fs.writeFileSync(path.join(output,'package.json'),JSON.stringify(pkg,null,2)+'\n',{flag:'wx'});
if(!fs.readFileSync(path.join(output,'schema.json')).equals(schemaBytes))throw Error('Legacy schema bytes changed');
const provenance={format:'elysium.compat-reader-derivation',revision,original:{name:original.name,version:original.version,
 schemaSha256:hash(schemaBytes),runtimeSha256:hash(Buffer.from(originalRuntime)),typesSha256:hash(fs.readFileSync(path.join(input,'index.d.ts')))},
 output:{version,schemaSha256:hash(fs.readFileSync(path.join(output,'schema.json'))),runtimeSha256:hash(fs.readFileSync(path.join(output,'index.cjs')))},
 generatorSha256:hash(fs.readFileSync(__filename)),validatorGeneratorSha256:hash(fs.readFileSync(path.join(__dirname,'generate.cjs'))),nativeSnapshotsRewritten:false};
fs.writeFileSync(path.join(output,'derivation.json'),JSON.stringify(provenance,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify(provenance));
