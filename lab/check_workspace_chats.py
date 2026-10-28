#!/usr/bin/env python3
"""Private, cross-process workspace chats; mock Codex, no model or external services."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import uuid

ROOT = Path(__file__).resolve().parent.parent
ARTIFACTS = ROOT / 'artifacts'


def main():
    ARTIFACTS.mkdir(exist_ok=True)
    for mode in ('save', 'restore', 'resume', 'resume-failure'):
        with tempfile.TemporaryDirectory(prefix='nexuterm-workspace-chats-') as temporary:
            base = Path(temporary)
            config = base / 'config/nexuterm'
            config.mkdir(parents=True)
            program = base / 'codex'
            fixture = (ROOT / 'lab/codex_fixture.py').read_text()
            if mode == 'resume-failure':
                fixture = fixture.replace("elif method == 'thread/resume':", "elif method == 'thread/resume':\n        emit(dict(id=ident,error=dict(code=-32000,message='fixture missing thread')));continue")
            program.write_text('#!/usr/bin/python3\n' + fixture)
            program.chmod(0o700)
            profiles = [dict(id='codex-fixture', name='Codex fixture', kind='Codex', program=str(program), directory=str(base), model='fixture-model')]
            (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[], ai_agents=profiles)))
            def chat(index):
                return dict(id=str(uuid.uuid4()), title=f'Chat {index}', agent_id='codex-fixture' if index == 0 else 'deleted-profile', model='fixture-model', directory=str(base), codex_thread='fixture-thread' if index == 0 else None,
                            messages=[dict(title='Tú', text='Pregunta áé\n```rust\nlet x = 1;\n```', tool=False, response=False), dict(title='Agente', text='Respuesta local sin conexión', tool=False, response=True), dict(title='Herramienta', text='resultado ficticio', tool=True, response=False)], draft=f'Borrador {index}\n')
            chats = [chat(i) for i in range(3)]
            def workspace(identifier, tabs):
                return dict(id=identifier, name=identifier, notes='', tabs=[], selected=0, directory=None, sidebar_visible=True, chats=dict(tabs=tabs, selected=1, visible=True))
            path = base / 'data/nexuterm/workspaces/workspaces.json'
            path.parent.mkdir(parents=True, mode=0o700)
            path.write_text(json.dumps(dict(version=1, workspaces=[workspace('Chats cliente', chats), workspace('Exceso', [chat(i) for i in range(6)])])))
            env = {k: v for k, v in os.environ.items() if not k.startswith('NEXUTERM_SMOKE') and k != 'NEXUTERM_SCREENSHOT'}
            env.update(HOME=temporary, XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / 'data'), XDG_STATE_HOME=str(base / 'state'), XDG_CACHE_HOME=str(base / 'cache'),
                       GDK_BACKEND='x11', GSK_RENDERER='cairo', SHELL='/bin/sh', NEXUTERM_SMOKE_TEST='1', NEXUTERM_SMOKE_WORKSPACE_CHATS=mode,
                       NEXUTERM_SCREENSHOT=str(ARTIFACTS / f'workspace-chats-{mode}.png'))
            result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')], cwd=base, env=env, capture_output=True, text=True, timeout=18)
            (ARTIFACTS / f'workspace-chats-{mode}.log').write_text(result.stdout + result.stderr)
            assert result.returncode == 0 and 'CRITICAL' not in result.stderr and 'panicked at' not in result.stderr, result.stderr
            marker = 'RESUME' if mode.startswith('resume') else mode.upper()
            assert f'NEXUTERM_SMOKE_CHATS_{marker}_OK' in result.stderr, result.stderr
            protocol = base / 'protocol.jsonl'
            if mode.startswith('resume'):
                requests = [json.loads(line) for line in protocol.read_text().splitlines()]
                resumed = [r for r in requests if r.get('method') == 'thread/resume']
                assert len(resumed) == 1 and resumed[0]['params']['threadId'] == 'fixture-thread'
                assert resumed[0]['params']['cwd'] == str(base)
                assert resumed[0]['params']['sandbox'] == 'read-only'
                assert not any(r.get('method') in ('thread/start', 'turn/start') for r in requests)
            else:
                assert not protocol.exists(), 'Restoring or saving must not launch the provider'
            if mode == 'save':
                saved = json.loads(path.read_text())['workspaces'][0]['chats']
                assert saved['tabs'][1]['draft'] == 'Borrador editado después de recuperar áé\n'
                assert saved['tabs'][2]['agent_id'] == 'deleted-profile'
                assert [c['id'] for c in saved['tabs']] == [c['id'] for c in chats]
                assert path.stat().st_mode & 0o777 == 0o600
                env['NEXUTERM_SMOKE_WORKSPACE_CHATS'] = 'restore'
                result2 = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')], cwd=base, env=env, capture_output=True, text=True, timeout=18)
                (ARTIFACTS / 'workspace-chats-reopen.log').write_text(result2.stdout + result2.stderr)
                assert result2.returncode == 0 and 'panicked at' not in result2.stderr and 'CRITICAL' not in result2.stderr, result2.stderr
                assert 'NEXUTERM_SMOKE_CHATS_RESTORE_OK' in result2.stderr, result2.stderr
                assert not protocol.exists()
            print(f'PASS workspace chats {mode}: offline contents, reuse, capacity and explicit-only resume')


if __name__ == '__main__':
    main()
