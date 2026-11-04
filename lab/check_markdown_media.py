#!/usr/bin/env python3
"""GTK tables and images with an isolated loopback HTTP fixture."""
import collections
import http.server
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import threading

ROOT = Path(__file__).resolve().parent.parent


def main():
    artifacts = ROOT / 'artifacts'
    artifacts.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='nx-markdown-media-') as directory:
        png = Path(directory) / 'image.png'
        counts = collections.Counter()
        closed = threading.Event()

        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def do_GET(self):
                counts[self.path] += 1
                try:
                    if self.path == '/missing':
                        self.send_error(404)
                        return
                    self.send_response(302 if self.path == '/redirect' else 200)
                    if self.path == '/redirect':
                        self.send_header('Location', 'file:///etc/hosts')
                    elif self.path == '/large':
                        self.send_header('Content-Length', str(5 * 1024 * 1024))
                    elif self.path == '/slow':
                        self.send_header('Content-Length', '1024')
                    self.end_headers()
                    self.wfile.flush()
                    if self.path == '/image.png':
                        self.wfile.write(png.read_bytes())
                    elif self.path == '/stream':
                        for _ in range(80):
                            self.wfile.write(b'x' * 65536)
                    elif self.path == '/slow':
                        self.connection.settimeout(3)
                        if self.connection.recv(1) == b'':
                            closed.set()
                except (BrokenPipeError, ConnectionResetError, socket.timeout):
                    pass

        server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        try:
            for theme in ('light', 'dark'):
                counts.clear()
                closed.clear()
                env = dict(os.environ, GDK_BACKEND='x11', GSK_RENDERER='cairo',
                           NEXUTERM_MARKDOWN_THEME=theme,
                           NEXUTERM_MEDIA_SCREENSHOT=str(artifacts / f'markdown-media-{theme}.png'),
                           NEXUTERM_MEDIA_HTTP=f'http://127.0.0.1:{server.server_port}',
                           NEXUTERM_MEDIA_HTTP_PNG=str(png))
                result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--',
                                         'cargo', 'test', '--locked', '--bin', 'nexuterm',
                                         'native_media_preserves', '--', '--ignored', '--test-threads=1'],
                                        cwd=ROOT, env=env, capture_output=True, text=True, timeout=180)
                log = result.stdout + result.stderr
                (artifacts / f'markdown-media-{theme}.log').write_text(log)
                assert result.returncode == 0 and '1 passed' in log and 'CRITICAL' not in log, log
                assert counts['/image.png'] == 1, counts
                assert all(counts[f'/{path}'] == 1 for path in ('missing', 'large', 'stream', 'redirect', 'slow')), counts
                assert closed.wait(2), 'HTTP request was not cancelled when the window closed'
                print(f'PASS {theme}: exact source, cells, structure, undo, local/explicit HTTP images, bounds, redirects and close cancellation')
        finally:
            server.shutdown()
            server.server_close()


if __name__ == '__main__':
    main()
