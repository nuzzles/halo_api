"""Serve Theater Lab's recording inspector over localhost using cached films.

Run from anywhere: python3 experiments/examples/theater_lab.py
Open http://127.0.0.1:8766. No authentication or external network requests.
"""

import argparse
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import RLock
from urllib.parse import parse_qs, urlsplit

from film_catalog import EXPERIMENTS_ROOT
from theater_inspector_data import Inspector

ASSETS = Path(__file__).parent / 'theater_inspector'


def handler_for(inspector):
    lock = RLock()  # Avoid duplicate large evidence loads on simultaneous requests.

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def send(self, content, content_type, status=200):
            self.send_response(status)
            self.send_header('Content-Type', content_type)
            self.send_header('Content-Length', str(len(content)))
            self.send_header('Cache-Control', 'no-store')
            self.send_header('X-Content-Type-Options', 'nosniff')
            self.end_headers()
            try:
                self.wfile.write(content)
            except (BrokenPipeError, ConnectionResetError):
                pass

        def do_GET(self):
            url = urlsplit(self.path)
            query = parse_qs(url.query)
            try:
                if url.path in ('/', '/inspector.js', '/inspector.css'):
                    name = {'/': 'index.html', '/inspector.js': 'inspector.js', '/inspector.css': 'inspector.css'}[url.path]
                    mime = {'html': 'text/html', 'js': 'text/javascript', 'css': 'text/css'}[name.rsplit('.', 1)[1]]
                    self.send((ASSETS / name).read_bytes(), mime + '; charset=utf-8')
                    return
                if url.path == '/recordings.js':
                    self.send((ASSETS.parent / 'recordings.js').read_bytes(), 'text/javascript; charset=utf-8')
                    return
                if url.path in ('/replay', '/theater_viewer.html'):
                    html = inspector.path('analysis/theater_viewer.html').read_bytes()
                    html = html.replace(b'href="http://127.0.0.1:8766/"', b'href="/"')
                    self.send(html, 'text/html; charset=utf-8')
                    return
                if url.path == '/api/decoded-film':
                    label = query.get('film', [''])[0]
                    if label not in inspector.catalog:
                        raise ValueError('Recording is not in the catalog')
                    file = inspector.path(label + '/decoded-film.json')
                    if not file.exists():
                        raise ValueError('This recording has not been decoded for replay yet.')
                    self.send(file.read_bytes(), 'application/json')
                    return
                if not url.path.startswith('/api/'):
                    self.send(b'Not found', 'text/plain', 404)
                    return
                with lock:
                    result = self.api(url.path, query)
                self.send(json.dumps(result, separators=(',', ':'), allow_nan=False).encode(), 'application/json')
            except (ValueError, KeyError, StopIteration, FileNotFoundError) as error:
                self.send(json.dumps({'error': str(error) or 'Item not found'}).encode(), 'application/json', 400)

        def api(self, route, query):
            def arg(name, default=None):
                if name not in query and default is None:
                    raise ValueError('Missing ' + name)
                return query.get(name, [default])[0]
            if route == '/api/catalog':
                rows = [dict(r, label=label) for label, r in inspector.catalog.items()
                        if inspector.path(label + '/film.json').exists()]
                return rows  # films.csv order is shared by both recording menus.
            label = arg('film')
            if label not in inspector.catalog:
                raise ValueError('Recording is not in the catalog')
            if route == '/api/film':
                return inspector.describe(label, refresh=arg('refresh', '0') == '1')
            if route == '/api/seek':
                time = float(arg('time'))
                if not 0 <= time <= inspector.describe(label)['duration']:
                    raise ValueError('Time is outside the film')
                return inspector.seek(label, time)
            index = int(arg('chunk'))
            if route == '/api/chunk':
                return inspector.chunk_info(label, index)
            if route == '/api/coverage':
                return inspector.chunk_coverage(label, index)
            if route == '/api/view':
                return inspector.view(label, index, int(arg('offset', '0')), int(arg('count', '256')))
            if route == '/api/search':
                _, data, _ = inspector.chunk(label, index)
                text, mode = arg('q'), arg('mode', 'text')
                needle = bytes.fromhex(text) if mode == 'hex' else text.encode('utf-8')
                if not needle or len(needle) > 256:
                    raise ValueError('Search requires 1–256 bytes')
                start = int(arg('start', '0'))
                if not 0 <= start <= len(data):
                    raise ValueError('Invalid search offset')
                offset = data.find(needle, start)
                wrapped = offset < 0
                if wrapped:
                    offset = data.find(needle, 0, start + len(needle) - 1)
                return {'offset': offset, 'length': len(needle), 'wrapped': wrapped and offset >= 0}
            raise ValueError('Unknown API route')

    return Handler


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--corpus', type=Path, default=EXPERIMENTS_ROOT / 'films')
    parser.add_argument('--port', type=int, default=8766)
    args = parser.parse_args()
    server = ThreadingHTTPServer(('127.0.0.1', args.port), handler_for(Inspector(args.corpus)))
    print(f'Theater Lab · Recording inspector: http://127.0.0.1:{args.port}', flush=True)
    print('Reads cached films only. Press Ctrl+C to stop.', flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == '__main__':
    main()
