/* Check historical identifiers against the offline official metadata catalog. */
const assert = require('node:assert/strict');
const { resolveArmor } = require('./theater_viewer/armor.js');
const catalog = require('./theater_viewer/armor-catalog.json');
const fixtures = require('../../src/theater/fixtures/appearance_records.json');
const captured = fixtures.records.find(r => r.film === 'weapons/05-ar-stalker-rifle').expected;
const named = Object.fromEntries(resolveArmor(captured, catalog).map(r => [r.kind, r.name]));
assert.deepEqual(named, {
  armor: 'Mark VII', coating: 'Cadet Brick', helmet: 'CAVALLINO', visor: 'Arcadian Green',
  gloves: 'Capaxx', knees: 'UA/Type SA', chest: 'TAC/Packrat Rig',
  left_shoulder: 'Alpha Augmentor', right_shoulder: 'Alpha Augmentor',
  wrist: 'TAC/Holodyne Milspec', utility: 'Myesel Ammo Pouch', mythic: 'Beyond the Burrow',
});
const old = { ...captured }; delete old.attachments; delete old.mythic_effect_ids;
for (const r of resolveArmor(old, catalog).slice(6)) { assert.equal(r.name, null); assert.equal(r.available, false); }
const unknown = { ...captured, regions: captured.regions.slice(1), attachments: { ...captured.attachments, chest_tag_id: 123 } };
const unresolved = resolveArmor(unknown, catalog);
assert.equal(unresolved.find(r => r.kind === 'gloves').name, null); // Both model regions must match.
assert.equal(unresolved.find(r => r.kind === 'chest').name, null);
const chest = catalog.items.find(r => r.name === 'TAC/Packrat Rig');
assert.equal(resolveArmor(captured, { items: [...catalog.items, { ...chest, name: 'Conflicting label' }] }).find(r => r.kind === 'chest').name, null);
// Zero tags in the old empty control cannot prove every geometry-based slot empty.
for (const r of resolveArmor(fixtures.records[0].expected, catalog).slice(6)) assert.equal(r.name, null);
console.log('PASS: recorded armor/attachment/mythic names, complete region matching, unavailable old fields, unknown identifiers and ambiguous names');
