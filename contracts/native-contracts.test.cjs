const assert = require('node:assert/strict');
const fs = require('node:fs');
const test = require('node:test');
const { encode } = require('@msgpack/msgpack');

test('targeted native returns, wireless copy and random infusion laws survive the public decoder',()=>{
  const {decodeTable}=require('./index.cjs');
  for(const name of ['container-planet','container-blackhole','wireless-upgrade','random-infusion-false','random-infusion-true']){
    const {recipe}=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/targeted-${name}-contract.json`,'utf8'));
    const table={kind:'recipes',records:[recipe]};
    assert.deepEqual(decodeTable(encode(table),'recipes'),table,name);
    if(recipe.process?.randomAspects){
      const bad=structuredClone(table);delete bad.records[0].process.randomAspects.bases;
      assert.throws(()=>decodeTable(encode(bad),'recipes'),/Invalid table/);
    }
    if(recipe.process?.kind==='wirelessTerminalUpgrade'){
      const bad=structuredClone(table);bad.records[0].process.wirelessVersion='unknown';
      assert.throws(()=>decodeTable(encode(bad),'recipes'),/Invalid table/);
    }
  }
});

test('production pool process and conditional output survive the public decoder',()=>{
  const {decodeTable}=require('./index.cjs');
  for(let i=0;i<4;i++){
    const fixture=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/botania-native-pool-${i}-contract.json`,'utf8'));
    for(const [kind,records]of [['recipes',[fixture.recipe]],['programs',fixture.programs]]){
      const table={kind,records};assert.deepEqual(decodeTable(encode(table),kind),table);
    }
    for(const key of ['scope','display','matching','index','program']){
      const changed=structuredClone(fixture.recipe);delete changed.process[key];
      assert.throws(()=>decodeTable(encode({kind:'recipes',records:[changed]}),'recipes'),/Invalid table/);
    }
  }
});

test('native bee pool shared chunks survive the public decoder with actual callback field names',()=>{
  const {decodeTable}=require('./index.cjs');
  const fixture=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/magicbees-pool-program.json`,'utf8'));
  const table={kind:'programs',records:fixture.chunks};
  assert.deepEqual(decodeTable(encode(table),'programs'),table);
  for(const [field,snake]of [['inputSpecies','input_species'],['outputSpecies','output_species'],['cachedOutput','cached_output'],['displayInput','display_input']]){
    const changed=structuredClone(table),entry=changed.records.find(v=>v.data.kind==='beePoolEntries').data.rows[0];
    entry[snake]=entry[field];delete entry[field];
    assert.throws(()=>decodeTable(encode(changed),'programs'),/Invalid table/);
  }
  for(const field of ['primary','secondary']){
    const changed=structuredClone(table),pair=changed.records.find(v=>v.data.kind==='beePoolForest').data.rows.find(Boolean);
    delete pair[field];assert.throws(()=>decodeTable(encode(changed),'programs'),/Invalid table/);
  }
  const missingTag=structuredClone(table);
  delete missingTag.records.find(v=>v.data.kind==='beePoolEntries').data.rows[0].prototype.nbt;
  assert.throws(()=>decodeTable(encode(missingTag),'programs'),/Invalid table/);
  const nativeCache=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/magicbees-pool-cache-transitions.json`,'utf8'))
    .observations[0].state.entries[0];
  // Decoder shape checks only; changed rows deliberately have no resealed content ID.
  for(const path of [['chromosome'],['chromosome','primary'],['chromosome','secondary']]){
    const changed=structuredClone(table),chunk=changed.records.find(v=>v.data.kind==='beePoolCacheEntries');
    chunk.data.rows=[structuredClone(nativeCache)];
    let target=chunk.data.rows[0];
    for(const key of path.slice(0,-1))target=target[key];
    delete target[path.at(-1)];assert.throws(()=>decodeTable(encode(changed),'programs'),/Invalid table/);
  }
});

test('GUI-free Forestry rows retain source-bound notes and shared rule chunks through the published decoder',()=>{
  const {decodeTable}=require('./index.cjs');
  for(const name of ['bottler','bottler-dynamic','fabricator','fermenter','moistener','carpenter']){
    const sample=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/forestry-${name}-semantic-contract.json`,'utf8'));
    const table={kind:'recipes',records:[sample.recipe]};
    assert.deepEqual(decodeTable(encode(table),'recipes'),table,name);
    assert.equal(sample.recipe.source.owner,'Forestry');
    assert.ok(sample.recipe.notes.length>0);
    for(const [kind,records]of [['programs',sample.programs??[]],['strings',sample.strings]]){
      const related={kind,records};assert.deepEqual(decodeTable(encode(related),kind),related,name);
      if(name==='fabricator'&&kind==='programs'){
        // Decoder-only shape probes; changing program contents invalidates its
        // content ID, so these are deliberately not full Domain witnesses.
        const wax=structuredClone(related);
        wax.records.find(row=>row.data.kind==='fabricatorRecipes').data.rows[0].planUse={kind:'wax',maxDamage:8,waxPlans:[]};
        assert.deepEqual(decodeTable(encode(wax),kind),wax);
        for(const field of ['maxDamage','waxPlans']){
          const invalid=structuredClone(wax),plan=invalid.records.find(row=>row.data.kind==='fabricatorRecipes').data.rows[0].planUse;
          plan[field==='maxDamage'?'max_damage':'wax_plans']=plan[field];delete plan[field];
          assert.throws(()=>decodeTable(encode(invalid),kind),/Invalid table/);
        }
      }
    }
  }
});

test('construction decoder retains explicit native display completion and rejects invented modes',()=>{
  const {decodeTable}=require('./index.cjs');
  const {records}=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/compat-structure-contract.json`,'utf8'));
  const build=structuredClone(records.find(row=>row.kind==='builds').value);
  build.completion='nativeDisplayBeforeControllerTick';
  const table={kind:'builds',records:[build]};
  assert.deepEqual(decodeTable(encode(table),'builds'),table);
  build.completion='skipAllTicks';
  assert.throws(()=>decodeTable(encode(table),'builds'),/Invalid table/);
});

test('new native callback and Botania contracts retain camelCase fields and required nullable targets',()=>{
  const {decodeTable}=require('./index.cjs');
  const gt=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/forestry-gt-container-observations.json`,'utf8'))[0].program.dynamic[0];
  const closed={kind:'noContainerRule',registry:'fixture:canister',drainedRegistries:['fixture:canister','fixture:empty'],readEffect:'preserve',minMeta:-2147483648,maxMeta:2147483647,compoundTags:[]};
  const table={kind:'programs',records:[{id:'program_fixture.chunk_fixture',program:'program_fixture',offset:0,data:{kind:'squeezerCallbacks',rows:[gt,closed]}}]};
  assert.deepEqual(decodeTable(encode(table),'programs'),table);
  const missing=structuredClone(table);delete missing.records[0].data.rows[0].rows[0].emptyRegistry;
  assert.throws(()=>decodeTable(encode(missing),'programs'),/Invalid table/);
  const snake=structuredClone(table);snake.records[0].data.rows[1].read_effect='preserve';delete snake.records[0].data.rows[1].readEffect;
  assert.throws(()=>decodeTable(encode(snake),'programs'),/Invalid table/);
  for(const stem of ['botania-mini-special','botania-mini-priority','botania-head-special','botania-head-priority']){
    const {recipe}=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/${stem}-contract.json`,'utf8'));
    assert.deepEqual(decodeTable(encode({kind:'recipes',records:[recipe]}),'recipes'),{kind:'recipes',records:[recipe]});
  }
});

test('generated usage contracts retain actual Java field names and explicit nullable provenance',()=>{
  const {decodeTable}=require('./index.cjs');
  const {domains}=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/native-usage-query-contract.json`,'utf8'));
  for(const records of domains){const table={kind:'references',records};assert.deepEqual(decodeTable(encode(table),'references'),table);}
  const records=structuredClone(domains[2]),doc=records.find(row=>row.data.kind==='usageDocument');
  delete doc.data.content.provider;
  assert.throws(()=>decodeTable(encode({kind:'references',records}),'references'),/Invalid table/);
});

test('published contracts validate in a fresh process without loading the runtime schema compiler',()=>{
  const {spawnSync}=require('node:child_process');
  const child=spawnSync(process.execPath,['-e',`
    const Module=require('node:module'),load=Module._load;
    Module._load=function(id,...args){if(id==='ajv')throw Error('Runtime schema compiler was loaded');return load.call(this,id,...args);};
    const api=require('./index.cjs'),sample=require('./fixtures/semantic-notes-contract.json');
    api.assertPointer({format:'elysium.catalog-pointer',revision:41,id:'a'.repeat(64)});
    api.assertTable({kind:'recipes',records:[sample.recipe]});
    const bad=structuredClone(sample.recipe);bad.invented=true;
    require('node:assert/strict').throws(()=>api.assertTable({kind:'recipes',records:[bad]}),/Invalid table/);
  `],{cwd:__dirname,encoding:'utf8',windowsHide:true});
  assert.equal(child.status,0,child.stderr);
});

test('canonical native process numbers match Java decimal identities', () => {
  const {canonical} = require('./index.cjs');
  assert.equal(canonical({z:-0,coords:[0.5,1,0.5],rate:1e-7}), '{"coords":[0.5,1,0.5],"rate":0.0000001,"z":0}');
  for(const family of ['altar','binding','meteor','charging']) {
    const {recipe}=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/blood-${family}-semantic-contract.json`,'utf8'));
    const key=Object.fromEntries(['source','category','inputs','duration','energy','grid','magic','process'].map(k=>[k,recipe[k]]));
    key.outputs=recipe.outputs;key.properties=Object.fromEntries(Object.entries(recipe.properties).map(([k,v])=>[k,v.value]));
    assert.equal('recipe_'+require('node:crypto').createHash('sha256').update(canonical(key)).digest('hex'),recipe.id);
  }
  for(const value of [NaN,Infinity,Number.MAX_SAFE_INTEGER+1,1e-101]) assert.throws(()=>canonical(value));
});

test('nested native machine tags accept actual rows and still reject foreign fields',()=>{
  const {decodeTable}=require('./index.cjs');
  for(const name of ['acclimatiser','sequencer','analyser','splicer','isolator','inoculator','genepool','incubator','polymeriser']){
    const {recipe}=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/binnie-genetics-${name}-semantic-contract.json`,'utf8'));
    assert.deepEqual(decodeTable(encode({kind:'recipes',records:[recipe]}),'recipes').records[0].process,recipe.process,name);
    recipe.process.foreignMachineField=1;
    assert.throws(()=>decodeTable(encode({kind:'recipes',records:[recipe]}),'recipes'),/Invalid table/);
  }
});

test('Java native notes pass the generated decoder without a GUI view', () => {
  const {decodeTable} = require('./index.cjs');
  const {recipe} = JSON.parse(fs.readFileSync(`${__dirname}/fixtures/semantic-notes-contract.json`, 'utf8'));
  assert.deepEqual(decodeTable(encode({kind:'recipes', records:[recipe]})).records[0].notes, recipe.notes);
  recipe.notes[0].invented = true;
  assert.throws(() => decodeTable(encode({kind:'recipes', records:[recipe]})), /Invalid table/);
});

test('current Catalog decoder rejects retired recipe GUI fields and tables', () => {
  const {decodeTable} = require('./index.cjs');
  const {recipe} = JSON.parse(fs.readFileSync(`${__dirname}/fixtures/semantic-notes-contract.json`, 'utf8'));
  recipe.view = 'view_native';
  assert.throws(() => decodeTable(encode({kind:'recipes',records:[recipe]})), /Invalid table/);
  assert.throws(() => decodeTable(encode({kind:'views',records:[{id:'view_native',width:16,height:16,elements:[]}]})), /Invalid table/);
});

test('semantic contracts expose current source and locale-bound catalog revisions', () => {
  const { formats, assertPointer } = require('./index.cjs');
  assert.equal(formats.source.revision, 41);
  assert.equal(formats.catalog.revision, 41);
  assertPointer({format: 'elysium.catalog-pointer', revision: 41, id: 'a'.repeat(64)});
  assert.throws(() => assertPointer({format: 'elysium.catalog-pointer', revision: 39, id: 'a'.repeat(64)}), /Invalid pointer/);
});

test('reference decoder preserves native villager offers without crafting them', () => {
  const { decodeTable } = require('./index.cjs');
  const { records } = JSON.parse(fs.readFileSync(`${__dirname}/fixtures/villager-trades-contract.json`, 'utf8'));
  const table = { kind: 'references', records };
  assert.deepEqual(decodeTable(encode(table), 'references').records, records);
  records[1].data.successChance = 1;
  assert.throws(() => decodeTable(encode(table), 'references'), /Invalid table/);
});

test('villager quantities retain signed native observations within Java int bounds', () => {
  const { decodeTable } = require('./index.cjs');
  const { records } = JSON.parse(fs.readFileSync(`${__dirname}/fixtures/villager-trades-contract.json`, 'utf8'));
  const table = {kind:'references', records};
  records[1].data.inputs[0].sizes = [-2147483648,-3,0,2,2147483647];
  assert.deepEqual(decodeTable(encode(table), 'references'), table);
  for (const bad of [-2147483649,2147483648,0.5]) {
    const invalid = structuredClone(table); invalid.records[1].data.inputs[0].sizes = [bad];
    assert.throws(() => decodeTable(encode(invalid), 'references'), /Invalid table/);
  }
});

test('decoder preserves conditional crop weights and rejects unknown predictor fields', () => {
  const { decodeTable } = require('./index.cjs');
  const { recipe } = JSON.parse(fs.readFileSync(`${__dirname}/fixtures/ic2-crop-prediction-contract.json`, 'utf8'));
  const table = { kind: 'recipes', records: [recipe] };
  assert.deepEqual(decodeTable(encode(table), 'recipes').records[0], recipe);
  recipe.process.invented = true;
  assert.throws(() => decodeTable(encode(table), 'recipes'), /Invalid table/);
});

test('generated decoder accepts native processor tables with finite float fields', () => {
  const { decodeTable } = require('./index.cjs');
  for (const name of ['blood-altar', 'blood-binding', 'blood-meteor', 'binnie-lumbermill-semantic',
    'forestry-bottler', 'forestry-bottler-dynamic', 'forestry-fabricator']) {
    const { recipe } = JSON.parse(fs.readFileSync(`${__dirname}/fixtures/${name}-contract.json`, 'utf8'));
    const decoded = decodeTable(encode({kind:'recipes',records:[recipe]}), 'recipes');
    assert.deepEqual(decoded.records[0].process, recipe.process, name);
  }
});

test('generated decoder preserves native action contracts', () => {
  const { decodeTable } = require('./index.cjs');
  const { recipe: cal } = JSON.parse(fs.readFileSync(`${__dirname}/fixtures/cal-imprint-contract.json`, 'utf8'));
  assert.deepEqual(decodeTable(encode({ kind: 'recipes', records: [cal] }), 'recipes').records[0], cal);
  for (const name of ['fmp-microblock', 'xu-microblock']) {
    const { recipe } = JSON.parse(fs.readFileSync(`${__dirname}/fixtures/${name}-contract.json`, 'utf8'));
    assert.deepEqual(decodeTable(encode({ kind: 'recipes', records: [recipe] }), 'recipes').records[0], recipe, name);
  }
  for (const name of ['space-pump','gas-siphon','gendustry-sampler','lootbag','lootbag-unresolved','banner-mask','banner-catalyst','banner-no-dye','extrautils-soul', 'ae-facade', ...Array.from({ length: 9 }, (_, index) => `ae-world-locale-${index}-en_US`),
    'gendustry-template', 'gendustry-mutatron', 'gendustry-replicator-0', 'gendustry-replicator-1', 'gendustry-replicator-2',
    'witchery-oven', ...['foul','exhale','hint','breath','none'].map(product => `witchery-oven-${product}`), 'witchery-kettle',
    'witchery-ritual', 'witchery-infernal', 'witchery-infernal-wildcard']) {
    const { recipe } = JSON.parse(fs.readFileSync(`${__dirname}/fixtures/${name}-contract.json`, 'utf8'));
    assert.deepEqual(decodeTable(encode({ kind: 'recipes', records: [recipe] }), 'recipes').records[0], recipe, name);
  }
});

test('generated decoder rejects nonfinite native upgrade rates', () => {
  const { decodeTable } = require('./index.cjs');
  const { recipe } = JSON.parse(fs.readFileSync(`${__dirname}/fixtures/blood-altar-contract.json`, 'utf8'));
  for (const value of [NaN, Infinity, -Infinity]) {
    recipe.process.upgrades.betterCapacityBase = value;
    assert.throws(() => decodeTable(encode({kind:'recipes',records:[recipe]}), 'recipes'), /Invalid table/);
  }
});

test('reference decoder preserves signed native tolerance bounds', () => {
  const { decodeTable } = require('./index.cjs');
  const fixture = JSON.parse(fs.readFileSync(`${__dirname}/fixtures/binnie-genetics-reference-rows-contract.json`, 'utf8'));
  const records = fixture.reference.records;
  const allele = records.find(row => row.data.kind === 'allele').data.allele;
  allele.toleranceBounds = [-128, 127];
  const table = { kind: 'references', records };
  assert.deepEqual(decodeTable(encode(table), 'references').records, records);
  for (const bounds of [[-129, 0], [0, 128], [-0.5, 1]]) {
    allele.toleranceBounds = bounds;
    assert.throws(() => decodeTable(encode(table), 'references'), /Invalid table/);
  }
});

test('grouped diagram decoder retains every native destination and rejects the retired single-target field',()=>{
  const {decodeTable}=require('./index.cjs');
  const {records}=JSON.parse(fs.readFileSync(`${__dirname}/fixtures/nei-diagram-grouped-contract.json`,'utf8'));
  const table={kind:'references',records};
  assert.deepEqual(decodeTable(encode(table),'references'),table);
  const binding=records.find(row=>row.data.kind==='neiDiagramBinding');
  assert.equal(binding.data.diagrams.length,120);
  binding.data.diagram=0;delete binding.data.diagrams;
  assert.throws(()=>decodeTable(encode(table),'references'),/Invalid table/);
});
