const fixtureSelect = document.querySelector('#fixture');
const chart = document.querySelector('#chart');
let report;

function formatMs(value) {
  return `${new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 }).format(value)} ms`;
}
function renderBar(value, maximum, parser) {
  const track = document.createElement('div');
  track.className = 'bar-track';
  track.setAttribute('aria-label', `${parser === 'halo' ? 'halo_api' : 'libfilmshell'} ${formatMs(value)}`);
  const fraction = value / maximum;
  const fill = document.createElement('span');
  fill.className = `bar ${parser}`;
  fill.style.width = `${fraction * 100}%`;
  const label = document.createElement('span');
  label.className = `bar-value${fraction < 0.28 ? ' outside' : ''}`;
  if (fraction < 0.28) label.style.left = `calc(${fraction * 100}% + 8px)`;
  label.textContent = formatMs(value);
  track.append(fill, label);
  return track;
}
function render() {
  const rows = fixtureSelect.value === 'all'
    ? report.results
    : report.results.filter(row => row.fixture === fixtureSelect.value);
  chart.replaceChildren();
  for (const row of rows) {
    const maximum = Math.max(row.halo_api_ms, row.libfilmshell_ms);
    const outer = document.createElement('div');
    outer.className = 'chart-row';
    const title = document.createElement('div');
    title.className = 'film-label';
    title.textContent = row.label;
    const detail = document.createElement('div');
    detail.className = 'film-detail';
    detail.textContent = `${row.chunks} chunks · ${row.compressed_mib.toFixed(2)} MiB · ${row.runs} runs`;
    title.append(detail);
    const bars = document.createElement('div');
    bars.className = 'bars';
    bars.append(renderBar(row.halo_api_ms, maximum, 'halo'));
    bars.append(renderBar(row.libfilmshell_ms, maximum, 'shell'));
    const lead = (1 - row.halo_api_ms / row.libfilmshell_ms) * 100;
    const badge = document.createElement('div');
    badge.className = `lead${lead < 0 ? ' negative' : ''}`;
    badge.textContent = `${lead >= 0 ? '+' : ''}${lead.toFixed(1)}%`;
    badge.title = 'Percent less time used by halo_api';
    outer.append(title, bars, badge);
    chart.append(outer);
  }
}
fetch('results.json')
  .then(response => {
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
  })
  .then(data => {
    if (data.schema !== 1 || !Array.isArray(data.results) || data.results.length === 0) throw new Error('Unexpected benchmark data');
    report = data;
    for (const row of data.results) fixtureSelect.add(new Option(row.label, row.fixture));
    const date = new Date(data.generated_at_unix_seconds * 1000).toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' });
    document.querySelector('#provenance').textContent = `Measured ${date} on ${data.os}/${data.arch}; halo_api ${data.halo_api_commit}, libfilmshell ${data.libfilmshell_commit.slice(0, 8)}. The parsers produce different output models.`;
    render();
  })
  .catch(error => { chart.textContent = `Results could not be loaded: ${error.message}`; });
fixtureSelect.addEventListener('change', render);
