const { decode } = require('@msgpack/msgpack');
const metadata = require('./metadata.json');
const validators = require('./validators.cjs');
if(validators.schemaSha256 !== metadata.schemaSha256) throw new Error('Generated contract validators do not match their schema metadata');
exports.formats = Object.freeze(Object.fromEntries(Object.entries(metadata.formats).map(([name,value])=>[name,Object.freeze(value)])));
exports.collections = Object.freeze(metadata.collections);
exports.topicKinds = Object.freeze(metadata.topicKinds);
const tableKinds = new Set(metadata.collections);

function validationError(name, errors) {
  const details = [...new Set((errors ?? []).map(error =>
    `${error.instancePath || '/'} ${error.message}`).map(message =>
    message.length > 256 ? message.slice(0, 253) + '...' : message))];
  const kept = details.slice(0, 6);
  const remaining = details.length - kept.length;
  return new Error(`Invalid ${name.toLowerCase()}: ${kept.join('; ')}${remaining ? `; ${remaining} additional validation details omitted` : ''}`);
}

function assert(name, value) {
  const validate = validators[name];
  if (!validate(value)) throw validationError(name, validate.errors);
}

function assertTable(value) {
  const kind = value !== null && typeof value === 'object' && !Array.isArray(value) ? value.kind : undefined;
  if (!tableKinds.has(kind)) throw new Error('Invalid table: unknown or missing kind');
  const validate = validators['table_' + kind];
  if (!validate(value)) throw validationError('Table', validate.errors);
}

exports.assertManifest = value => assert('Manifest', value);
exports.assertPointer = value => assert('Pointer', value);
exports.assertTable = assertTable;
exports.decodeTable = (bytes, kind) => {
  if (!(bytes instanceof Uint8Array) || bytes.length > 16 * 1024 * 1024) throw new Error('Catalog table exceeds 16 MiB');
  const table = decode(bytes, { maxStrLength: 1024 * 1024, maxBinLength: 0,
    maxArrayLength: 1000000, maxMapLength: 131072, maxExtLength: 0, useBigInt64: true });
  assertTable(table);
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
    if (!Number.isFinite(value) || Number.isInteger(value) && !Number.isSafeInteger(value))
      throw new Error('JSON quantities must be exact integer strings; native decimals must be finite');
    if(value===0)return '0';
    const negative=value<0;
    const [mantissa,exponent='0']=String(Math.abs(value)).toLowerCase().split('e');
    const [integer,fraction='']=mantissa.split('.');
    const leading=(integer+fraction).replace(/^0+/,'');
    const digits=leading.replace(/0+$/,'');
    const scale=fraction.length-Number(exponent)-(leading.length-digits.length);
    if(digits.length>100||Math.abs(scale)>100)throw new Error('JSON number exceeds native canonical range');
    const point=digits.length-scale;
    const decimal=scale<=0?digits+'0'.repeat(-scale):point<=0?'0.'+'0'.repeat(-point)+digits:digits.slice(0,point)+'.'+digits.slice(point);
    return (negative?'-':'')+decimal;
  }
  if (typeof value === 'string' || typeof value === 'boolean') return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(entry => canonical(entry, depth + 1)).join(',')}]`;
  if (typeof value === 'object') return `{${Object.keys(value).sort(compare).map(key =>
    `${JSON.stringify(key)}:${canonical(value[key], depth + 1)}`).join(',')}}`;
  throw new Error('Unsupported canonical JSON value');
}
exports.canonical = canonical;
