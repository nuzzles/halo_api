/* Names come from official item metadata; equipment comes only from Film samples. */
(function (root) {
  'use strict';
  const slots = [['armor', 'Armor base'], ['coating', 'Coating'], ['helmet', 'Helmet'],
    ['visor', 'Visor'], ['gloves', 'Gloves'], ['knees', 'Knee pads'], ['chest', 'Chest'],
    ['left_shoulder', 'Left shoulder'], ['right_shoulder', 'Right shoulder'],
    ['wrist', 'Wrist'], ['utility', 'Utility'], ['mythic', 'Mythic effect']];
  const attachmentKinds = new Set(['chest', 'left_shoulder', 'right_shoulder', 'wrist', 'utility']);
  function resolveArmor(value, catalog) {
    const regions = new Map((value.regions || []).map(r => [r.region_id, r.permutation_id]));
    return slots.map(([kind, label]) => {
      const matches = catalog.items.filter(item => item.kind === kind &&
        Object.entries(item.match).every(([key, expected]) => {
          if (key === 'regions') return expected.every(r => regions.get(r.region_id) === r.permutation_id);
          if (key === 'attachments') return value.attachments && Object.entries(expected).every(([field, id]) => value.attachments[field] === id);
          if (key === 'mythic_effect_ids') return Array.isArray(value[key]) && value[key].length === expected.length && expected.every((id, i) => value[key][i] === id);
          return value[key] === expected;
        }));
      const names = [...new Set(matches.map(item => item.name))];
      const available = attachmentKinds.has(kind) ? value.attachments != null : kind === 'mythic' ? value.mythic_effect_ids != null : true;
      return { kind, label, available, name: names.length === 1 ? names[0] : null,
        paths: names.length === 1 ? matches.map(item => item.path) : [] };
    });
  }
  if (typeof module !== 'undefined' && module.exports) module.exports = { resolveArmor };
  else root.resolveArmor = resolveArmor;
})(typeof globalThis !== 'undefined' ? globalThis : this);
