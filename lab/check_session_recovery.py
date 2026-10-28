#!/usr/bin/env python3
"""Automatic checkpoints with real GTK/SIGKILL and private disposable data."""
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import uuid

ROOT = Path(__file__).resolve().parent.parent
ARTIFACTS = ROOT / 'artifacts'


def seed(base):
    config = base / 'config/nexuterm'
    config.mkdir(parents=True)
    (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[])))
    (base / 'document.txt').write_text('Original sin modificar\n')
    chat = dict(id=str(uuid.uuid4()), title='Chat sin conexión', agent_id='deleted-profile', model='fixture-model', directory=str(base), codex_thread=None,
                messages=[dict(title='Tú', text='Pregunta anterior áé', tool=False, response=False), dict(title='Agente', text='Respuesta anterior', tool=False, response=True)], draft='Borrador inicial\n')
    (base / 'chat-seed.json').write_text(json.dumps(dict(tabs=[chat], selected=0, visible=True)))


def run(base, mode, crash=False):
    env = {k: v for k, v in os.environ.items() if not k.startswith('NEXUTERM_SMOKE') and k != 'NEXUTERM_SCREENSHOT'}
    env.update(HOME=str(base), XDG_CONFIG_HOME=str(base / 'config'), XDG_DATA_HOME=str(base / 'data'), XDG_CACHE_HOME=str(base / 'cache'), XDG_STATE_HOME=str(base / 'state'),
               GDK_BACKEND='x11', GSK_RENDERER='cairo', SHELL='/bin/sh', NEXUTERM_SMOKE_TEST='1', NEXUTERM_SMOKE_SESSION=mode,
               NEXUTERM_SCREENSHOT=str(ARTIFACTS / f'session-{mode}.png'))
    path = ARTIFACTS / f'session-{mode}.log'
    with path.open('w+') as log:
        process = subprocess.Popen(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')], cwd=base, env=env, stdout=log, stderr=log, start_new_session=True)
        try:
            if crash:
                deadline = time.monotonic() + 10
                while time.monotonic() < deadline:
                    output = path.read_text()
                    if 'NEXUTERM_SMOKE_SESSION_CAPTURE_OK' in output:
                        pid = int(next(s.split('=', 1)[1] for s in output.splitlines() if s.startswith('NEXUTERM_SESSION_PID=')))
                        os.kill(pid, signal.SIGKILL)
                        assert process.wait(timeout=5) != 0
                        break
                    assert process.poll() is None, output
                    time.sleep(.05)
                else:
                    raise AssertionError(path.read_text())
            else:
                assert process.wait(timeout=18) == 0, path.read_text()
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=5)
    output = path.read_text()
    assert 'panicked at' not in output and 'CRITICAL' not in output, output
    marker = 'CLOSE_REQUESTED' if mode == 'close' else mode.upper().replace('-', '_') + '_OK'
    assert 'NEXUTERM_SMOKE_SESSION_' + marker in output, output


def snapshots(base):
    return sorted((base / 'data/nexuterm/workspaces/sessions').glob('*.json'))


def main():
    ARTIFACTS.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='nexuterm-auto-session-') as temporary:
        base = Path(temporary)
        seed(base)
        run(base, 'capture', crash=True)
        copies = snapshots(base)
        assert len(copies) == 1
        old = copies[0]
        workspace = json.loads(old.read_text())['workspaces'][0]
        assert workspace['tabs'][0]['layout']['kind'] == 'Split'
        assert workspace['chats']['tabs'][0]['draft'] == 'Último borrador de chat sin enviar áé\n'
        assert not (base / 'data/nexuterm/workspaces/workspaces.json').exists(), 'No manually saved workspace should be needed'
        assert old.stat().st_mode & 0o777 == 0o600
        assert old.parent.stat().st_mode & 0o777 == 0o700
        drafts = list((base / 'data/nexuterm/recovery').glob('*.json'))
        assert len(drafts) == 1 and json.loads(drafts[0].read_text())['text'] == 'Última edición automática áé\n'
        run(base, 'restore-failure')
        assert not old.exists(), 'Old copy must be retired after durable replacement'
        assert len(snapshots(base)) == 1
        restored = snapshots(base)[0]
        preserved_draft = drafts[0].read_bytes()
        run(base, 'discard')
        assert not restored.exists()
        assert drafts[0].read_bytes() == preserved_draft
        assert (base / 'document.txt').read_text() == 'Original sin modificar\n'
        print('PASS automatic session: two periodic captures, SIGKILL, startup recovery, disconnected splits/chat, failed handoff, retry, durable replacement and explicit discard')
    for mode in ('close', 'failure', 'invalid'):
        with tempfile.TemporaryDirectory(prefix='nexuterm-auto-' + mode + '-') as temporary:
            base = Path(temporary)
            seed(base)
            damaged = None
            if mode == 'invalid':
                directory = base / 'data/nexuterm/workspaces/sessions'
                directory.mkdir(parents=True)
                damaged = directory / (str(uuid.uuid4()) + '.json')
                damaged.write_text('unknown-format')
            run(base, mode)
            assert (base / 'document.txt').read_text() == 'Original sin modificar\n'
            if mode == 'close':
                assert len(snapshots(base)) == 1
                saved = json.loads(snapshots(base)[0].read_text())['workspaces'][0]
                assert saved['chats']['tabs'][0]['draft'] == 'Borrador inicial\n'
                assert json.loads(next((base / 'data/nexuterm/recovery').glob('*.json')).read_text())['text'] == 'Primera edición automática áé\n'
            if damaged:
                assert damaged.read_text() == 'unknown-format' and len(snapshots(base)) == 2
            print(f'PASS automatic session {mode}: originals and previous data preserved')


if __name__ == '__main__':
    main()
