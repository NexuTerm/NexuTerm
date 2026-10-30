#!/usr/bin/env python3
"""UI continuation, pending attachment/reasoning and full local history; synthetic providers."""
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
    ARTIFACTS.mkdir(exist_ok=True)
    for mode in ('api-continue', 'acp-continue', 'acp-retry', 'acp-disconnect', 'codex-state', 'history-many'):
        calls = []
        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass
            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
                calls.append(body)
                assert len(body['input']) == 3
                assert body['input'][0] == dict(role='user', content='primero')
                assert body['input'][1] == dict(role='assistant', content='respuesta-1')
                assert body['input'][2]['content'].startswith('segundo mensaje de prueba')
                assert 'missing.txt' in body['input'][2]['content']
                answer = json.dumps(dict(status='completed', output=[dict(type='message', content=[dict(type='output_text', text='respuesta-2')])])).encode()
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(answer)))
                self.end_headers()
                self.wfile.write(answer)
        server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        worker = threading.Thread(target=server.serve_forever, daemon=True)
        worker.start()
        try:
            with tempfile.TemporaryDirectory(prefix='nexuterm-chat-state-') as temporary:
                base = Path(temporary)
                config = base / 'config/nexuterm'
                config.mkdir(parents=True)
                program = base / ('acp' if mode in ('acp-continue', 'acp-retry', 'acp-disconnect') else 'codex')
                fixture = ROOT / ('lab/acp_fixture.py' if mode in ('acp-continue', 'acp-retry', 'acp-disconnect') else 'lab/codex_fixture.py')
                program.write_text('#!/usr/bin/python3\n' + fixture.read_text())
                program.chmod(0o700)
                if mode in ('acp-continue', 'acp-retry', 'acp-disconnect'):
                    (base / 'support-load').touch()
                    if mode in ('acp-retry', 'acp-disconnect'):
                        (base / ('load-disconnect' if mode == 'acp-disconnect' else 'load-failure')).touch()
                profile = dict(id='codex-fixture', name='Fixture', kind='Acp' if mode in ('acp-continue', 'acp-retry', 'acp-disconnect') else 'Codex', program=str(program), args=[], endpoint='', directory=temporary, model='fixture/two' if mode in ('acp-continue', 'acp-retry', 'acp-disconnect') else 'fixture-model', terminal_access=False)
                if mode == 'api-continue':
                    profile.update(kind='Responses', program='', endpoint=f'http://127.0.0.1:{server.server_port}/v1')
                (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[], ai_agents=[profile])))
                destination = hashlib.sha256(json.dumps([profile['id'], profile['kind'], profile['program'], profile['args'], profile['endpoint']], separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
                continuation = (dict(kind='Api', destination=destination, messages=[dict(role='user', content='primero'), dict(role='assistant', content='respuesta-1')]) if mode == 'api-continue' else dict(kind='Acp', destination=destination, session_id='fixture-session') if mode in ('acp-continue', 'acp-retry', 'acp-disconnect') else None)
                def chat(index):
                    return dict(id=str(uuid.uuid4()), title=f'Chat {index}', agent_id='codex-fixture' if index == 0 else 'deleted-profile', model=profile['model'], directory=temporary, codex_thread='fixture-thread' if index == 0 and mode in ('codex-state', 'history-many') else None,
                                messages=[dict(title='Tú', text='Pregunta', tool=False, response=False), dict(title='Agente', text='Respuesta local', tool=False, response=True), dict(title='Herramienta', text='Salida conservada', tool=True, response=False)], draft='Borrador sin enviar',
                                attachments=[str(base / 'missing.txt'), str(base)] if index == 0 else [], reasoning='high' if index == 0 and mode in ('codex-state', 'history-many') else '', continuation=continuation if index == 0 else None)
                def workspace(name, tabs):
                    return dict(id=name, name=name, notes='', tabs=[], selected=0, directory=None, sidebar_visible=True, chats=dict(tabs=tabs, selected=1, visible=True))
                path = base / 'data/nexuterm/workspaces/workspaces.json'
                path.parent.mkdir(parents=True)
                path.write_text(json.dumps(dict(version=1, workspaces=[workspace('Chats', [chat(i) for i in range(3)]), workspace('Exceso', [chat(i) for i in range(6)])])))
                env = {k: v for k, v in os.environ.items() if not k.startswith('NEXUTERM_SMOKE') and k != 'NEXUTERM_SCREENSHOT'}
                env.update(HOME=temporary, XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / 'data'), XDG_STATE_HOME=str(base / 'state'), XDG_CACHE_HOME=str(base / 'cache'),
                           GDK_BACKEND='x11', GSK_RENDERER='cairo', SHELL='/bin/sh', NEXUTERM_SMOKE_TEST='1', NEXUTERM_SMOKE_WORKSPACE_CHATS=mode,
                           NEXUTERM_SCREENSHOT=str(ARTIFACTS / f'chat-state-{mode}.png'),
                           NEXUTERM_CHAT_HISTORY_SCREENSHOT=str(ARTIFACTS / f'chat-history-{mode}.png'))
                result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')], cwd=base, env=env, capture_output=True, text=True, timeout=30)
                (ARTIFACTS / f'chat-state-{mode}.log').write_text(result.stdout + result.stderr)
                assert result.returncode == 0 and 'panicked at' not in result.stderr and 'CRITICAL' not in result.stderr, result.stderr
                assert 'NEXUTERM_SMOKE_CHAT_CONTINUATION_OK' in result.stderr, result.stderr
                assert len(calls) == (1 if mode == 'api-continue' else 0)
                if mode in ('acp-continue', 'acp-retry', 'acp-disconnect'):
                    protocol = [json.loads(line) for line in (base / 'acp-protocol.jsonl').read_text().splitlines()]
                    assert not any(r.get('method') == 'session/new' for r in protocol)
                    assert sum(r.get('method') == 'session/load' for r in protocol) == (2 if mode in ('acp-retry', 'acp-disconnect') else 1)
                    assert any(r.get('id') == 'replay-permission' and r.get('result', {}).get('outcome') == dict(outcome='cancelled') for r in protocol)
                if mode in ('codex-state', 'history-many'):
                    protocol = [json.loads(line) for line in (base / 'protocol.jsonl').read_text().splitlines()]
                    turns = [r for r in protocol if r.get('method') == 'turn/start']
                    assert len(turns) == (75 if mode == 'history-many' else 1)
                    assert all(r['params']['effort'] == 'high' for r in turns)
                assert not (base / 'missing.txt').exists(), 'Restoring attachments must not recreate or read missing files'
                print(f'PASS chat {mode}: explicit continuation, preserved state and complete local archive')
        finally:
            server.shutdown()
            server.server_close()
            worker.join(timeout=2)


if __name__ == '__main__':
    main()
