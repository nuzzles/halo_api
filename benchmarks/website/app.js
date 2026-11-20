const chart = document.querySelector('#chart');

function formatMs(value) {
  return `${new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 }).format(value)} ms`;
}

function renderBar(value, maximum, parser) {
  const line = document.createElement('div');
  line.className = 'bar-line';
  const names = { halo: 'halo_api', shell: 'libfilmshell', jgtm: 'JGtm (partial decode)' };
  line.setAttribute('aria-label', `${names[parser]}: ${formatMs(value)}`);
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
    const maximum = Math.max(row.halo_api_ms, row.libfilmshell_ms, row.jgtm_partial_ms);
    const outer = document.createElement('div');
    outer.className = 'chart-row';
    const title = document.createElement('div');
    title.className = 'film-label';
    title.textContent = row.label;
    const detail = document.createElement('span');
    detail.className = 'film-detail';
    detail.textContent = `${row.runs} trials · ${row.chunks} chunks · ${row.compressed_mib.toFixed(2)} MiB`;
    title.append(detail);
    const bars = document.createElement('div');
    bars.className = 'bars';
    bars.append(renderBar(row.halo_api_ms, maximum, 'halo'));
    bars.append(renderBar(row.libfilmshell_ms, maximum, 'shell'));
    bars.append(renderBar(row.jgtm_partial_ms, maximum, 'jgtm'));
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
    if (report.schema !== 2 || !Array.isArray(report.results) || report.results.length === 0 ||
        report.results.some(row => !Number.isFinite(row.jgtm_partial_ms))) {
      throw new Error('Unexpected benchmark data');
    }
    document.querySelector('#cpu-model').textContent = report.cpu_model || `${report.os}/${report.arch}`;
    document.querySelector('#threading').textContent = report.threading || 'Unspecified';
    const trials = [...new Set(report.results.map(row => row.runs))].sort((a, b) => a - b);
    const trialCount = new Intl.ListFormat(undefined, { type: 'disjunction' }).format(trials.map(String));
    document.querySelector('#chart-title').textContent =
      `Median parse time of ${trialCount} ${trials.length === 1 && trials[0] === 1 ? 'trial' : 'trials'}`;
    const updated = document.querySelector('#updated');
    const date = new Date(report.generated_at_unix_seconds * 1000);
    const day = date.toISOString().slice(0, 10);
    document.querySelector('#dataset-date').replaceChildren(new Option(day, day));
    updated.dateTime = date.toISOString();
    updated.textContent = `Updated ${date.toLocaleString(undefined, {
      year: 'numeric', month: 'long', day: 'numeric', hour: 'numeric', minute: '2-digit', timeZoneName: 'short'
    })}`;
    render(report.results);
  })
  .catch(error => { chart.textContent = `Results could not be loaded: ${error.message}`; });
