#!/usr/bin/env python3
"""Recent-chat navigation, continuation and isolation, with loopback providers."""
import hashlib
import http.server
import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import uuid

ROOT = Path(__file__).resolve().parent.parent
ARTIFACTS = ROOT / 'artifacts'


def main():
    calls = []

    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            calls.append(body)
            answer = json.dumps(dict(status='completed', output=[dict(type='message', content=[dict(type='output_text', text=f'respuesta-{len(calls)}')])])).encode()
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Length', str(len(answer)))
            self.end_headers()
            self.wfile.write(answer)

    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    worker = threading.Thread(target=server.serve_forever, daemon=True)
    worker.start()
    try:
        ARTIFACTS.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='nexuterm-recents-') as temporary:
            base = Path(temporary)
            config = base / 'config/nexuterm'
            config.mkdir(parents=True)
            directory = base / 'data/nexuterm/chat-history'
            directory.mkdir(parents=True, mode=0o700)
            profiles = []
            for name, identifier in [('Codex', 'codex-fixture'), ('GitHub', 'github-fixture')]:
                profile = dict(id=identifier, name=name, kind='Responses', program='', args=[], endpoint=f'http://127.0.0.1:{server.server_port}/v1', directory=temporary, model='fixture-model', terminal_access=False)
                profiles.append(profile)
                destination = hashlib.sha256(json.dumps([identifier, profile['kind'], '', [], profile['endpoint']], separators=(',', ':')).encode()).hexdigest()
                chat = dict(id=str(uuid.uuid4()), title=f'{name} anterior', agent_id=identifier, model='fixture-model', directory=temporary, codex_thread=None,
                            messages=[dict(title='Tú', text=f'Pregunta {name}', tool=False, response=False), dict(title=name, text=f'Respuesta {name}', tool=False, response=True)], draft=f'Borrador {name}',
                            continuation=dict(kind='Api', destination=destination, messages=[dict(role='user', content=f'Pregunta {name}'), dict(role='assistant', content=f'Respuesta {name}')]))
                path = directory / f"{chat['id']}.json"
                path.write_text(json.dumps(dict(version=1, captured_at=1, chat=chat)))
                path.chmod(0o600)
            (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[], ai_agents=profiles, preferences=dict(theme='dark'))))
            for phase in ('initial', 'reload', 'native'):
                if phase == 'native':
                    program = base / 'codex'
                    program.write_text('#!/usr/bin/python3\n' + (ROOT / 'lab/codex_fixture.py').read_text())
                    program.chmod(0o700)
                    native = dict(id='codex-native', name='Codex', kind='Codex', program=str(program), args=[], endpoint='', directory=temporary, model='fixture-model', terminal_access=False)
                    (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[], ai_agents=[native])))
                env = {k: v for k, v in os.environ.items() if not k.startswith('NEXUTERM_SMOKE') and k != 'NEXUTERM_SCREENSHOT'}
                env.update(HOME=temporary, XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / 'data'), XDG_STATE_HOME=str(base / 'state'), XDG_CACHE_HOME=str(base / 'cache'),
                           GDK_BACKEND='x11', GSK_RENDERER='cairo', SHELL='/bin/sh', NEXUTERM_SMOKE_TEST='1', NEXUTERM_SMOKE_RECENTS=phase,
                           NEXUTERM_RECENTS_SCREENSHOT=str(ARTIFACTS / f'chat-recents-{phase}.png'))
                result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')], cwd=base, env=env, capture_output=True, text=True, timeout=30)
                (ARTIFACTS / f'chat-recents-{phase}.log').write_text(result.stdout + result.stderr)
                assert result.returncode == 0 and 'panicked at' not in result.stderr and 'CRITICAL' not in result.stderr, result.stderr
                assert 'NEXUTERM_SMOKE_RECENTS_OK' in result.stderr, result.stderr
            assert len(calls) == 3, calls
            assert len(calls[0]['input']) == 1, 'Fresh typing must not reuse earlier context'
            for call, name in zip(calls[1:], ('Codex', 'GitHub')):
                assert call['input'][0] == dict(role='user', content=f'Pregunta {name}')
                assert call['input'][1] == dict(role='assistant', content=f'Respuesta {name}')
                assert call['input'][2]['content'].startswith(f'Continuar {name}')
            snapshots = [json.loads(p.read_text())['chat'] for p in directory.glob('*.json')]
            snapshots = [chat for chat in snapshots if chat['agent_id'] != 'codex-native']
            protocol = [json.loads(line) for line in (base / 'protocol.jsonl').read_text().splitlines()]
            assert sum(r.get('method') == 'thread/start' for r in protocol) == 1
            assert sum(r.get('method') == 'thread/resume' for r in protocol) == 2
            assert any(r.get('method') == 'thread/list' and r['params'].get('cursor') == 'cursor-2' for r in protocol)
            assert len(snapshots) == 3, snapshots
            assert all(p.stat().st_mode & 0o777 == 0o600 for p in directory.glob('*.json'))
            assert directory.stat().st_mode & 0o777 == 0o700
            print('PASS recents: full chat view, same tab, fresh input, continuation, per-agent history, restart and native Codex pagination')
    finally:
        server.shutdown()
        server.server_close()
        worker.join(timeout=2)


if __name__ == '__main__':
    main()
