const chart = document.querySelector('#chart');

function formatMs(value) {
  return `${new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 }).format(value)} ms`;
}

function renderBar(value, maximum, parser) {
  const line = document.createElement('div');
  line.className = 'bar-line';
  line.setAttribute('aria-label', `${parser === 'halo' ? 'halo_api' : 'libfilmshell'}: ${formatMs(value)}`);
  const rail = document.createElement('span');
  rail.className = 'bar-rail';
  const fill = document.createElement('span');
  fill.className = `bar ${parser}`;
  fill.style.width = `${value / maximum * 100}%`;
  rail.append(fill);
  const label = document.createElement('span');
  label.className = 'bar-value';
  label.textContent = formatMs(value);
  line.append(rail, label);
  return line;
}

function render(results) {
  chart.replaceChildren();
  for (const row of results) {
    const maximum = Math.max(row.halo_api_ms, row.libfilmshell_ms);
    const outer = document.createElement('div');
    outer.className = 'chart-row';
    const title = document.createElement('div');
    title.className = 'film-label';
    title.textContent = row.label;
    const detail = document.createElement('span');
    detail.className = 'film-detail';
    detail.textContent = `${row.chunks} chunks · ${row.compressed_mib.toFixed(2)} MiB`;
    title.append(detail);
    const bars = document.createElement('div');
    bars.className = 'bars';
    bars.append(renderBar(row.halo_api_ms, maximum, 'halo'));
    bars.append(renderBar(row.libfilmshell_ms, maximum, 'shell'));
    outer.append(title, bars);
    chart.append(outer);
  }
}

fetch('results.json')
  .then(response => {
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
  })
  .then(report => {
    if (report.schema !== 1 || !Array.isArray(report.results) || report.results.length === 0) {
      throw new Error('Unexpected benchmark data');
    }
    document.querySelector('#cpu-model').textContent = report.cpu_model || `${report.os}/${report.arch}`;
    document.querySelector('#threading').textContent = report.threading || 'Unspecified';
    const updated = document.querySelector('#updated');
    const date = new Date(report.generated_at_unix_seconds * 1000);
    updated.textContent = `Updated ${date.toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' })}`;
    render(report.results);
  })
  .catch(error => { chart.textContent = `Results could not be loaded: ${error.message}`; });
