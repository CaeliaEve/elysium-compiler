const Ajv = require('ajv');
const { decode } = require('@msgpack/msgpack');
const schema = require('./schema.json');
exports.formats = Object.freeze(Object.fromEntries([['source', 'SourceManifest'], ['catalog', 'Manifest']].map(([name, type]) => {
  const fields = schema.definitions[type].properties;
  return [name, Object.freeze({ name: fields.format.enum[0], revision: fields.revision.enum[0] })];
})));
exports.collections = Object.freeze(schema.definitions.Table.oneOf.map(table => table.properties.kind.enum[0]).sort());
exports.topicKinds = Object.freeze(schema.definitions.TopicKind.enum);

const ajv = new Ajv({ strict: true, allowUnionTypes: true });
for (const [name, minimum, maximum] of [
  ['uint8', 0, 255], ['uint32', 0, 4294967295], ['int32', -2147483648, 2147483647],
  ['uint64', 0, Number.MAX_SAFE_INTEGER],
]) ajv.addFormat(name, { type: 'number', validate: value =>
  Number.isSafeInteger(value) && value >= minimum && value <= maximum });
ajv.addSchema(schema, 'elysium');
const validators = Object.fromEntries(['Manifest', 'Pointer', 'Table'].map(name =>
  [name, ajv.compile({ $ref: `elysium#/definitions/${name}` })]));

function assert(name, value) {
  const validate = validators[name];
  if (!validate(value)) throw new Error(`Invalid ${name.toLowerCase()}: ${ajv.errorsText(validate.errors)}`);
}

exports.assertManifest = value => assert('Manifest', value);
exports.assertPointer = value => assert('Pointer', value);
exports.assertTable = value => assert('Table', value);
exports.decodeTable = (bytes, kind) => {
  if (!(bytes instanceof Uint8Array) || bytes.length > 16 * 1024 * 1024) throw new Error('Catalog table exceeds 16 MiB');
  const table = decode(bytes, { maxStrLength: 1024 * 1024, maxBinLength: 0,
    maxArrayLength: 1000000, maxMapLength: 131072, maxExtLength: 0, useBigInt64: true });
  assert('Table', table);
  if (kind !== undefined && table.kind !== kind) throw new Error('Catalog table kind does not match its manifest');
  return table;
};

const encoder = new TextEncoder();
function compare(left, right) {
  const a = encoder.encode(left), b = encoder.encode(right);
  for (let index = 0; index < Math.min(a.length, b.length); index++) {
    if (a[index] !== b[index]) return a[index] - b[index];
  }
  return a.length - b.length;
}

function canonical(value, depth = 0) {
  if (depth > 64) throw new Error('Canonical JSON exceeds nesting limit');
  if (value === null) return 'null';
  if (typeof value === 'number') {
    if (!Number.isSafeInteger(value)) throw new Error('JSON quantities must be exact integer strings');
    return String(value);
  }
  if (typeof value === 'string' || typeof value === 'boolean') return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(entry => canonical(entry, depth + 1)).join(',')}]`;
  if (typeof value === 'object') return `{${Object.keys(value).sort(compare).map(key =>
    `${JSON.stringify(key)}:${canonical(value[key], depth + 1)}`).join(',')}}`;
  throw new Error('Unsupported canonical JSON value');
}
exports.canonical = canonical;
