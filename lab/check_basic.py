#!/usr/bin/env python3
"""Real Telnet client inside GTK against disposable loopback test servers."""
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import threading

ROOT = Path(__file__).resolve().parent.parent


def check(kind):
    events, failures = [], []
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        listener.listen(1)
        listener.settimeout(12)
        port = listener.getsockname()[1]

        def serve():
            try:
                connection, _ = listener.accept()
                with connection:
                    connection.settimeout(8)
                    stream = connection.makefile('rwb', buffering=0)
                    stream.write(b'NEXUTERM_BASIC_OK\r\n')
                    assert b'hello-lab' in stream.readline()
                    events.append('input')
            except Exception as error:
                failures.append(repr(error))

        thread = threading.Thread(target=serve, daemon=True)
        thread.start()
        with tempfile.TemporaryDirectory(prefix='nexuterm-basic-') as directory:
            config = Path(directory) / 'nexuterm'
            config.mkdir()
            (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[])))
            env = {key: value for key, value in os.environ.items()
                   if not key.startswith('NEXUTERM_SMOKE') and key != 'NEXUTERM_SCREENSHOT'}
            env.update(XDG_DATA_HOME=str(Path(directory) / 'data'), HOME=directory, XDG_CONFIG_HOME=directory, NEXUTERM_SMOKE_TEST='1',
                       NEXUTERM_SMOKE_BASIC=f'{kind}:{port}', GSK_RENDERER='cairo',
                       NEXUTERM_SCREENSHOT=str(ROOT / 'artifacts' / f'{kind}-basic.png'))
            command = [str(ROOT / 'target/debug/nexuterm')]
            result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--', *command], cwd=ROOT,
                                    env=env, text=True, capture_output=True, timeout=20)
            thread.join(timeout=10)
            artifacts = ROOT / 'artifacts'
            artifacts.mkdir(exist_ok=True)
            (artifacts / f'{kind}-basic.log').write_text(result.stdout + result.stderr)
            assert result.returncode == 0, f'{kind}: app failed'
            assert 'NEXUTERM_SMOKE_BASIC_OK' in result.stderr, f'{kind}: no terminal marker'
            assert not thread.is_alive() and not failures, (kind, failures)
            assert events == ['input'], events
            print(f'PASS {kind}: real client embedded in GTK, loopback authentication/input')


if __name__ == '__main__':
    check('telnet')
