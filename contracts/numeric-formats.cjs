// Shared by generation and the standalone validators. Keep native numeric bounds exact.
for(const [name,minimum,maximum] of [
 ['uint8',0,255],['uint16',0,65535],['uint32',0,4294967295],
 ['int8',-128,127],['int16',-32768,32767],['int32',-2147483648,2147483647],
 ['uint64',0,Number.MAX_SAFE_INTEGER],
])exports[name]={type:'number',validate:value=>Number.isSafeInteger(value)&&value>=minimum&&value<=maximum};
exports.double={type:'number',validate:Number.isFinite};
exports.float={type:'number',validate:value=>Number.isFinite(value)&&Number.isFinite(Math.fround(value))};
