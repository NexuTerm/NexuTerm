#!/usr/bin/env python3
"""Explicit per-chat access through GTK, fake CLIs and loopback API tools."""
import http.server
import json
import shutil
import tempfile
import threading
from pathlib import Path
from check_ai import ROOT, ARTIFACTS, run


def check_cli(kind):
    with tempfile.TemporaryDirectory(prefix='nexuterm-full-access-') as temporary:
        directory = Path(temporary)
        codex = kind == 'Codex'
        cli = directory / ('codex' if codex else 'acp')
        cli.write_text('#!' + shutil.which('python3') + '\n' + (ROOT / 'lab' / ('codex_fixture.py' if codex else 'acp_fixture.py')).read_text())
        cli.chmod(0o700)
        (directory / 'allow-permission').touch()
        profile = dict(id='full-' + kind, name=kind, kind=kind, program=str(cli), directory=temporary, model='fixture-model' if codex else 'fixture/one')
        _, logs = run('full-access', profile, directory)
        assert 'NEXUTERM_SMOKE_FULL_ACCESS_OK' in logs, logs
        protocol = directory / ('protocol.jsonl' if codex else 'acp-protocol.jsonl')
        requests = [json.loads(line) for line in protocol.read_text().splitlines()]
        if codex:
            start = next(r['params'] for r in requests if r.get('method') == 'thread/start')
            assert start['sandbox'] == 'danger-full-access' and start['approvalPolicy'] == 'never'
            assert any(r.get('id') == 'approval-1' and r.get('result', {}).get('decision') == 'accept' for r in requests)
        else:
            assert any(r.get('id') == 'permission' and r.get('result', {}).get('outcome') == dict(outcome='selected', optionId='yes') for r in requests)
        print(f'PASS {kind}: GTK full access, real protocol approval and revocation')


def check_api(kind):
    with tempfile.TemporaryDirectory(prefix='nexuterm-full-api-') as temporary:
        directory = Path(temporary)
        requests = []
        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass
            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
                requests.append(body)
                responses = kind == 'Responses'
                if len(requests) == 1:
                    arguments = json.dumps(dict(target_id='local', directory=temporary, command='printf API; touch api-executed', keep_history=False))
                    if responses:
                        reply = dict(output=[dict(type='function_call', name='supervised_execute', arguments=arguments, call_id='tool-1')])
                    else:
                        reply = dict(choices=[dict(message=dict(role='assistant', content=None, tool_calls=[dict(id='tool-1', type='function', function=dict(name='supervised_execute', arguments=arguments))]))])
                else:
                    if responses:
                        reply = dict(output=[dict(type='message', content=[dict(type='output_text', text='Ejecutado')])])
                    else:
                        reply = dict(choices=[dict(message=dict(role='assistant', content='Ejecutado'))])
                raw = json.dumps(reply).encode()
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(raw)))
                self.end_headers()
                self.wfile.write(raw)
        server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            profile = dict(id='full-' + kind, name=kind, kind=kind, endpoint=f'http://127.0.0.1:{server.server_port}/v1', model='fixture-model', terminal_access=True)
            _, logs = run('full-access', profile, directory)
            assert 'NEXUTERM_SMOKE_FULL_ACCESS_OK' in logs, logs
            assert (directory / 'api-executed').exists()
            assert len(requests) == 2
            messages = requests[1]['input' if kind == 'Responses' else 'messages']
            tool = next(m for m in messages if m.get('type') == 'function_call_output' or m.get('role') == 'tool')
            result = json.loads(tool.get('output', tool.get('content')))
            assert not result['isError']
            evidence = json.loads(result['content'][0]['text'])
            assert evidence['stdout'] == 'API' and evidence['exit_code'] == 0
            print(f'PASS {kind}: GTK full access, supervised tool executed without dialog and real evidence returned')
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=2)


if __name__ == '__main__':
    ARTIFACTS.mkdir(exist_ok=True)
    for kind in ('Codex', 'Acp'):
        check_cli(kind)
    for kind in ('Responses', 'ChatCompletions'):
        check_api(kind)
