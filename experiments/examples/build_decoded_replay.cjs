/* Build the offline toy from typed Film JSON, using the same adapter as import. */
const fs = require('node:fs');
const path = require('node:path');
const { gzipSync } = require('node:zlib');
const { filmToClip } = require('./theater_viewer/film.js');
const { normalize } = require('./recordings.js');
const assets = path.join(__dirname, 'theater_viewer');
let output = path.resolve(__dirname, '../films/analysis/theater_viewer.html');
let files = [];
let embed = false, explicitOutput = false, corpusRequested = false, octagonWalls = false;
let fontDirectory = null, fixedLoadout = null, replayData = null;
const availableLabels = new Set();
const args = process.argv.slice(2);
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--embed') embed = true;
  else if (args[i] === '--octagon-walls') octagonWalls = true;
  else if (args[i] === '--fixed-loadout') {
    fixedLoadout = (args[++i] || '').split(',').map(name => name.trim());
    if (fixedLoadout.length !== 2 || fixedLoadout.some(name => !name) || fixedLoadout[0] === fixedLoadout[1]) throw new Error('--fixed-loadout requires two distinct weapon names in slot order');
  }
  else if (args[i] === '--blog-fonts') { if (!args[++i]) throw new Error('Missing --blog-fonts directory'); fontDirectory = path.resolve(args[i]); }
  else if (args[i] === '--output') { if (!args[++i]) throw new Error('Missing --output path'); output = path.resolve(args[i]); explicitOutput = true; }
  else if (args[i] === '--corpus') {
    corpusRequested = true;
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
if (!files.length) throw new Error('Usage: node build_decoded_replay.cjs <decoded-film.json>... [--embed | --corpus <folder>] [--output <html>]');
if (embed && (corpusRequested || files.length !== 1)) throw new Error('--embed requires exactly one decoded-film.json and does not accept --corpus');
if (octagonWalls && !embed) throw new Error('--octagon-walls requires --embed');
if (fontDirectory && !embed) throw new Error('--blog-fonts requires --embed');
if (fixedLoadout && !embed) throw new Error('--fixed-loadout requires --embed');
if (embed && !explicitOutput) output = path.resolve(__dirname, '../films/analysis/motion_replay_embed.html');
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
const catalog = embed ? new Map() : readCatalog(path.resolve(__dirname, '../films.csv'));
const decodedIds = new Set();
const replayErrors = new Map();
const clips = [...new Set(files)].map(file => {
  const source = fs.readFileSync(file);
  const film = JSON.parse(source);
  if (embed) replayData = source;
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
if (octagonWalls) clips[0].placeholderArena = 'octagon';
if (fixedLoadout) clips[0].fixedLoadout = fixedLoadout;
const recordings = normalize([...catalog.values()].filter(row => decodedIds.has(row.match_id) || availableLabels.has(`${row.group}/${row.slug}`)));
for (const row of recordings) if (replayErrors.has(row.match_id)) row.replay_error = replayErrors.get(row.match_id);
for (const clip of clips) if (!recordings.some(row => row.id === clip.id)) recordings.push(...normalize([clip]));
let html = fs.readFileSync(path.join(assets, 'index.html'), 'utf8');
if (embed) {
  const title = clips[0].name.replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);
  html = html.replace(/<!-- LAB_ONLY_START -->[\s\S]*?<!-- LAB_ONLY_END -->/g, '')
    .replace('<body>', '<body class="replay-embed">')
    .replace('<head>', '<head>\n  <meta name="robots" content="noindex">')
    .replace('<title>Theater Lab · Motion replay</title>', `<title>${title} · Motion replay</title>`)
    .replace('<a id="inspect-recording">Open this recording in the inspector →</a>', '')
    .replace('<span>↕ Scroll to zoom</span>', '<span>Ctrl/⌘ + scroll to zoom</span>');
}
const replacements = {
  __VIEWER_CSS__: fs.readFileSync(path.join(assets, 'viewer.css'), 'utf8') + (embed ? '\n' + fs.readFileSync(path.join(assets, 'embed.css'), 'utf8') : '') + (fontDirectory ?
    '\n/*\n' + fs.readFileSync(path.join(fontDirectory, 'OFL.txt'), 'utf8').replaceAll('*/', '* /') + '\n*/\n' + [400, 600].map(weight =>
      `@font-face{font-family:Barlow;font-weight:${weight};font-style:normal;font-display:swap;src:url(data:font/woff2;base64,${fs.readFileSync(path.join(fontDirectory, `barlow-${weight}.woff2`)).toString('base64')}) format("woff2");}`).join('\n') : ''),
  __THREE_JS__: '/*\n' + fs.readFileSync(path.join(assets, 'vendor/LICENSE.three'), 'utf8') + '\n*/\n' + fs.readFileSync(path.join(assets, 'vendor/three.min.js'), 'utf8'),
  __CLIP_DATA__: JSON.stringify(clips).replaceAll('<', '\\u003c'),
  __RECORDING_DATA__: JSON.stringify(embed ? [] : recordings).replaceAll('<', '\\u003c'),
  __RECORDINGS_JS__: embed ? '' : fs.readFileSync(path.join(__dirname, 'recordings.js'), 'utf8'),
  __FILM_IMPORT_JS__: embed ? '' : fs.readFileSync(path.join(assets, 'film.js'), 'utf8'),
  __ARMOR_CATALOG__: JSON.stringify(JSON.parse(fs.readFileSync(path.join(assets, 'armor-catalog.json'), 'utf8'))).replaceAll('<', '\\u003c'),
  __ARMOR_JS__: fs.readFileSync(path.join(assets, 'armor.js'), 'utf8'),
  __SPARTAN_JS__: embed ? fs.readFileSync(path.join(assets, 'spartan.js'), 'utf8') : '',
  __VIEWER_JS__: '/*\n' + fs.readFileSync(path.resolve(__dirname, '../../LICENSE-MIT'), 'utf8') + '\n*/\n' + fs.readFileSync(path.join(assets, 'viewer.js'), 'utf8'),
};
for (const [token, value] of Object.entries(replacements)) html = html.replace(token, () => value);
fs.mkdirSync(path.dirname(output), { recursive: true }); fs.writeFileSync(output, html);
if (embed) {
  const dataOutput = path.join(path.dirname(output), path.basename(output, path.extname(output)) + '.json.gz');
  fs.writeFileSync(dataOutput, gzipSync(replayData));
}
console.log(`Built ${output}: ${recordings.length} recordings, ${clips.length} replayable films, ${clips.reduce((n, c) => n + c.players.length, 0)} player tracks`);
