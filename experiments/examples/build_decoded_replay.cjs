/* Build the offline toy from typed Film JSON, using the same adapter as import. */
const fs = require('node:fs');
const path = require('node:path');
const { filmToClip } = require('./theater_viewer/film.js');
const { normalize } = require('./recordings.js');
const assets = path.join(__dirname, 'theater_viewer');
let output = path.resolve(__dirname, '../films/analysis/theater_viewer.html');
let files = [];
const availableLabels = new Set();
const args = process.argv.slice(2);
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--output') { if (!args[++i]) throw new Error('Missing --output path'); output = path.resolve(args[i]); }
  else if (args[i] === '--corpus') {
    if (!args[++i]) throw new Error('Missing --corpus path');
    const corpus = path.resolve(args[i]);
    for (const group of fs.readdirSync(corpus, { withFileTypes: true }).filter(d => d.isDirectory())) {
      for (const film of fs.readdirSync(path.join(corpus, group.name), { withFileTypes: true }).filter(d => d.isDirectory())) {
        if (fs.existsSync(path.join(corpus, group.name, film.name, 'film.json'))) availableLabels.add(`${group.name}/${film.name}`);
        const file = path.join(corpus, group.name, film.name, 'decoded-film.json');
        if (fs.existsSync(file)) files.push(file);
      }
    }
  } else if (args[i].startsWith('-')) throw new Error(`Unknown argument ${args[i]}`);
  else files.push(path.resolve(args[i]));
}
if (!files.length) throw new Error('Usage: node build_decoded_replay.cjs <decoded-film.json>... [--corpus <folder>] [--output <html>]');
// Catalog titles/categories override older cached sidecars.
function readCatalog(file) {
  const rows = []; let row = [], cell = '', quoted = false;
  const text = fs.readFileSync(file, 'utf8');
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === '"') {
      if (quoted && text[i + 1] === '"') { cell += '"'; i++; }
      else quoted = !quoted;
    } else if (!quoted && (c === ',' || c === '\n')) {
      row.push(cell); cell = '';
      if (c === '\n') { if (row.some(Boolean)) rows.push(row); row = []; }
    } else if (quoted || c !== '\r') cell += c;
  }
  if (quoted) throw new Error('Unclosed quote in film catalog');
  if (cell || row.length) { row.push(cell); rows.push(row); }
  const headers = rows.shift() || [];
  return new Map(rows.map(values => {
    if (values.length !== headers.length) throw new Error('Incomplete film catalog row');
    const entry = Object.fromEntries(headers.map((key, i) => [key, values[i]]));
    return [entry.match_id, entry];
  }));
}
const catalog = readCatalog(path.resolve(__dirname, '../films.csv'));
const decodedIds = new Set();
const replayErrors = new Map();
const clips = [...new Set(files)].map(file => {
  const film = JSON.parse(fs.readFileSync(file, 'utf8'));
  decodedIds.add(film.match_id);
  const sidecar = path.join(path.dirname(file), 'film.json');
  const metadata = fs.existsSync(sidecar) ? JSON.parse(fs.readFileSync(sidecar, 'utf8')) : {};
  try { return filmToClip(film, { ...metadata, ...catalog.get(film.match_id) }); }
  catch (error) {
    if (error.code !== 'NO_DECODED_POSITIONS') throw error;
    replayErrors.set(film.match_id, error.message);
    console.warn(`Skipped ${metadata.group || ''}/${metadata.slug || film.match_id}: ${error.message} See ${file} for decoder diagnostics.`);
    return null;
  }
}).filter(Boolean).sort((a, b) => b.duration - a.duration || a.id.localeCompare(b.id));
if (!clips.length) throw new Error('No films contain decoded positions; existing replay was not overwritten.');
const recordings = normalize([...catalog.values()].filter(row => decodedIds.has(row.match_id) || availableLabels.has(`${row.group}/${row.slug}`)));
for (const row of recordings) if (replayErrors.has(row.match_id)) row.replay_error = replayErrors.get(row.match_id);
for (const clip of clips) if (!recordings.some(row => row.id === clip.id)) recordings.push(...normalize([clip]));
let html = fs.readFileSync(path.join(assets, 'index.html'), 'utf8');
const replacements = {
  __VIEWER_CSS__: fs.readFileSync(path.join(assets, 'viewer.css'), 'utf8'),
  __THREE_JS__: fs.readFileSync(path.join(assets, 'vendor/three.min.js'), 'utf8'),
  __CLIP_DATA__: JSON.stringify(clips).replaceAll('<', '\\u003c'),
  __RECORDING_DATA__: JSON.stringify(recordings).replaceAll('<', '\\u003c'),
  __RECORDINGS_JS__: fs.readFileSync(path.join(__dirname, 'recordings.js'), 'utf8'),
  __FILM_IMPORT_JS__: fs.readFileSync(path.join(assets, 'film.js'), 'utf8'),
  __ARMOR_CATALOG__: JSON.stringify(JSON.parse(fs.readFileSync(path.join(assets, 'armor-catalog.json'), 'utf8'))).replaceAll('<', '\\u003c'),
  __ARMOR_JS__: fs.readFileSync(path.join(assets, 'armor.js'), 'utf8'),
  __VIEWER_JS__: fs.readFileSync(path.join(assets, 'viewer.js'), 'utf8'),
};
for (const [token, value] of Object.entries(replacements)) html = html.replace(token, () => value);
fs.mkdirSync(path.dirname(output), { recursive: true }); fs.writeFileSync(output, html);
console.log(`Built ${output}: ${recordings.length} recordings, ${clips.length} replayable films, ${clips.reduce((n, c) => n + c.players.length, 0)} player tracks`);
