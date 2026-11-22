const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { test } = require('node:test');
const vm = require('node:vm');

class Element {
  children = [];
  style = {};
  attributes = {};
  dataset = {};
  events = {};
  text = '';
  set textContent(value) { this.text = value; this.children = []; }
  get textContent() { return this.text; }
  setAttribute(name, value) { this.attributes[name] = value; }
  append(...children) { this.children.push(...children); }
  replaceChildren(...children) { this.text = ''; this.children = children; }
  addEventListener(name, listener) { this.events[name] = listener; }
  click() { this.events.click(); }
}

function report(benchmark, threading) {
  return {
    schema: 1, benchmark, threading: `${threading === 'multi' ? 'Multi' : 'Single'}-Threaded`,
    generated_at_unix_seconds: 1795248000, halo_api_commit: 'example', halo_api_dirty: true,
    cpu_model: 'Test CPU',
    results: [{ label: benchmark, chunks: 2, compressed_mib: 1, runs: 25, halo_api_ms: 5,
      ...(benchmark === 'full' && threading === 'multi' ? { libfilmshell_ms: 10 } : {}) }],
  };
}

const pathFor = (benchmark, threading) => `results${benchmark === 'full' ? '' : `-${benchmark}`}${threading === 'multi' ? '' : '-single'}.json`;
const flush = () => new Promise(resolve => setImmediate(resolve));

function app(fetchReport = async (path, benchmark, threading) => report(benchmark, threading)) {
  const elements = new Map();
  const groups = {};
  for (const [group, names] of Object.entries({ threading: ['multi', 'single'], benchmark: ['full', 'replication', 'summary'] })) {
    groups[group] = names.map((name, index) => {
      const element = new Element();
      element.dataset[group] = name;
      element.attributes['aria-pressed'] = String(index === 0);
      return element;
    });
  }
  const element = id => {
    if (!elements.has(id)) elements.set(id, new Element());
    return elements.get(id);
  };
  const calls = [];
  const context = vm.createContext({
    document: {
      querySelector: element,
      querySelectorAll: selector => groups[selector.split(' ')[0].slice(1)],
      createElement: () => new Element(),
    },
    Option: class { constructor(text, value) { this.text = text; this.value = value; } },
    fetch: async path => {
      calls.push(path);
      const benchmark = path.includes('replication') ? 'replication' : path.includes('summary') ? 'summary' : 'full';
      const threading = path.includes('single') ? 'single' : 'multi';
      const data = await fetchReport(path, benchmark, threading);
      return { ok: true, json: async () => data };
    },
  });
  vm.runInContext(readFileSync(`${__dirname}/app.js`, 'utf8'), context);
  return { element, calls, groups, click: (group, name) => groups[group].find(button => button.dataset[group] === name).click() };
}

test('all six selections load the correct report, legend, and pressed states', async () => {
  const page = app();
  await flush();
  for (const benchmark of ['full', 'replication', 'summary']) {
    page.click('benchmark', benchmark);
    for (const threading of ['multi', 'single']) {
      page.click('threading', threading);
      await flush();
      assert.ok(page.calls.includes(pathFor(benchmark, threading)));
      assert.equal(page.element('#chart').children[0].children[0].textContent, benchmark);
      assert.equal(page.element('#shell-legend').hidden, benchmark !== 'full' || threading !== 'multi');
      assert.equal(page.element('#chart-title').textContent, 'Median parse time of 25 trials');
      for (const group of ['benchmark', 'threading']) {
        const selected = group === 'benchmark' ? benchmark : threading;
        for (const button of page.groups[group]) {
          assert.equal(button.attributes['aria-pressed'], String(button.dataset[group] === selected));
        }
      }
    }
  }
  assert.equal(page.calls.length, 6, 'successful reports are cached');
});

test('late responses cannot replace the selected benchmark', async () => {
  let resolveFull;
  const page = app((path, benchmark, threading) => benchmark === 'full'
    ? new Promise(resolve => { resolveFull = resolve; }) : report(benchmark, threading));
  page.click('benchmark', 'summary');
  await flush();
  resolveFull(report('full', 'multi'));
  await flush();
  assert.equal(page.element('#chart').children[0].children[0].textContent, 'summary');
});

test('bars retain the original per-film scale', async () => {
  const page = app(async (path, benchmark, threading) => {
    const data = report(benchmark, threading);
    data.results.push({ ...data.results[0], label: 'Slower', halo_api_ms: 10 });
    return data;
  });
  page.click('benchmark', 'replication');
  await flush();
  const rows = page.element('#chart').children;
  const fill = row => row.children[1].children[0].children[0].children[0];
  assert.equal(fill(rows[0]).style.width, '100%');
  assert.equal(fill(rows[1]).style.width, '100%');
});

test('invalid scope data is rejected without blocking other views', async () => {
  const page = app(async (path, benchmark, threading) => report(benchmark === 'replication' ? 'full' : benchmark, threading));
  await flush();
  page.click('benchmark', 'replication');
  await flush();
  assert.match(page.element('#chart').textContent, /Unexpected benchmark label/);
  page.click('benchmark', 'summary');
  await flush();
  assert.equal(page.element('#chart').children.length, 1);
});

test('legacy full reports work, but partial libfilmshell comparisons are rejected', async () => {
  const page = app(async (path, benchmark, threading) => {
    const data = report(benchmark, threading);
    if (benchmark === 'full') delete data.benchmark;
    else data.results[0].libfilmshell_ms = 10;
    return data;
  });
  await flush();
  assert.equal(page.element('#chart').children.length, 1);
  page.click('benchmark', 'summary');
  await flush();
  assert.match(page.element('#chart').textContent, /Unexpected parser results/);
});

test('published datasets render all views and agree across threading modes', async () => {
  const reports = new Map();
  for (const benchmark of ['full', 'replication', 'summary']) {
    for (const threading of ['multi', 'single']) {
      const path = pathFor(benchmark, threading);
      reports.set(path, JSON.parse(readFileSync(`${__dirname}/${path}`, 'utf8')));
      assert.ok(reports.get(path).results.every(row => row.runs === 25), `${path} must contain 25-trial measurements`);
    }
    const multi = reports.get(pathFor(benchmark, 'multi'));
    const single = reports.get(pathFor(benchmark, 'single'));
    assert.deepEqual(multi.results.map(row => [row.fixture, row.film_fingerprint]),
      single.results.map(row => [row.fixture, row.film_fingerprint]));
  }
  const page = app(async path => reports.get(path));
  await flush();
  for (const benchmark of ['full', 'replication', 'summary']) {
    page.click('benchmark', benchmark);
    for (const threading of ['multi', 'single']) {
      page.click('threading', threading);
      await flush();
      assert.equal(page.element('#chart').children.length, reports.get(pathFor(benchmark, threading)).results.length);
      assert.equal(page.element('#chart').textContent, '');
    }
  }
});

test('website retains compact benchmark choices without extra explanatory copy', () => {
  const html = readFileSync(`${__dirname}/index.html`, 'utf8');
  const css = readFileSync(`${__dirname}/styles.css`, 'utf8');
  const script = readFileSync(`${__dirname}/app.js`, 'utf8');
  assert.doesNotMatch(html, /benchmark-description|id="revision"/);
  assert.doesNotMatch(script, /local changes|Includes compressed-input copying/);
  assert.match(css, /\.benchmark-options\s*\{[^}]*display:\s*flex;[^}]*flex-wrap:\s*wrap;/);
  assert.match(css, /\.benchmark-options button\s*\{[^}]*flex:\s*0 0 auto;/);
});
