/* Shared recording menu for the offline replay and both live Theater Lab views. */
(function (root) {
  'use strict';
  function normalize(rows) {
    return rows.map(row => ({ ...row,
      id: row.label || row.id || (row.group && row.slug ? `${row.group}/${row.slug}` : row.match_id),
      title: row.title?.trim() || row.name || row.slug || row.match_id,
      category: row.category || row.group || 'Decoded films',
    }));
  }
  function populate(select, rows) {
    const groups = new Map();
    select.replaceChildren();
    for (const row of normalize(rows)) {
      if (!groups.has(row.category)) {
        const group = document.createElement('optgroup'); group.label = row.category;
        groups.set(row.category, group); select.append(group);
      }
      const option = document.createElement('option');
      option.value = row.id; option.textContent = row.title;
      groups.get(row.category).append(option);
    }
  }
  const api = { normalize, populate };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  else root.TheaterRecordings = api;
})(typeof globalThis !== 'undefined' ? globalThis : this);
