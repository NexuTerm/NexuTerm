#!/usr/bin/env python3
"""Isolated GTK approval, local execution and MCP lifecycle. No production commands."""
import json
import os
from pathlib import Path
import subprocess
import signal
import time
import tempfile
ROOT = Path(__file__).resolve().parent.parent

def main():
    artifacts = ROOT / 'artifacts'
    artifacts.mkdir(exist_ok=True)
    for theme in ('dark', 'light'):
        with tempfile.TemporaryDirectory(prefix='nexuterm-supervised-') as temporary:
            base = Path(temporary)
            config = base / 'config/nexuterm'
            config.mkdir(parents=True)
            (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[], preferences=dict(theme=theme))))
            env = {k: v for k, v in os.environ.items() if not k.startswith('NEXUTERM_SMOKE') and k != 'NEXUTERM_SCREENSHOT'}
            env.update(HOME=temporary, XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / 'data'), XDG_STATE_HOME=str(base / 'state'), XDG_CACHE_HOME=str(base / 'cache'), SHELL='/bin/sh', GDK_BACKEND='x11', GSK_RENDERER='cairo', NEXUTERM_SMOKE_TEST='1', NEXUTERM_SMOKE_SUPERVISED='1', NEXUTERM_SUPERVISED_SCREENSHOT=str(artifacts / f'supervised-{theme}.png'))
            result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')], env=env, cwd=base, capture_output=True, text=True, timeout=30)
            (artifacts / f'supervised-{theme}.log').write_text(result.stdout + result.stderr)
            assert result.returncode == 0 and 'panicked at' not in result.stderr and 'CRITICAL' not in result.stderr, result.stderr
            assert 'NEXUTERM_SMOKE_SUPERVISED_OK' in result.stderr, result.stderr
            records = list((base / 'data/nexuterm/executions').glob('*.json'))
            assert len(records) == 1, records
            record = json.loads(records[0].read_text())
            assert record['exit_code'] == 7 and bytes(record['stdout']) == 'salida á'.encode()
            assert records[0].stat().st_mode & 0o777 == 0o600
            assert records[0].parent.stat().st_mode & 0o777 == 0o700
            assert not list(records[0].parent.glob('*.reserve'))
            print(f'PASS supervised {theme}: reject/approve, cwd, streams, exit, timeout, cancel, private history, exclusion and MCP expiry')

    with tempfile.TemporaryDirectory(prefix='nexuterm-execution-crash-') as temporary:
        base = Path(temporary)
        config = base / 'config/nexuterm'
        config.mkdir(parents=True)
        (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[])))
        env = {k: v for k, v in os.environ.items() if not k.startswith('NEXUTERM_SMOKE') and k != 'NEXUTERM_SCREENSHOT'}
        env.update(HOME=temporary, XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / 'data'), XDG_STATE_HOME=str(base / 'state'), XDG_CACHE_HOME=str(base / 'cache'), SHELL='/bin/sh', GDK_BACKEND='x11', GSK_RENDERER='cairo', NEXUTERM_SMOKE_TEST='1', NEXUTERM_SMOKE_SUPERVISED='capture')
        capture_log = artifacts / 'supervised-crash.log'
        with capture_log.open('w') as log:
            process = subprocess.Popen(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')], env=env, cwd=base, stdout=log, stderr=log)
            try:
                for _ in range(150):
                    lines = capture_log.read_text().splitlines()
                    pid = next((int(line.split('=')[1]) for line in lines if line.startswith('NEXUTERM_EXECUTION_PID=')), None)
                    if pid is not None:
                        os.kill(pid, signal.SIGKILL)
                        break
                    assert process.poll() is None, capture_log.read_text()
                    time.sleep(.05)
                else:
                    raise AssertionError(capture_log.read_text())
                process.wait(timeout=10)
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=5)
        env['NEXUTERM_SMOKE_SUPERVISED'] = 'recover'
        result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')], env=env, cwd=base, capture_output=True, text=True, timeout=25)
        (artifacts / 'supervised-recovery.log').write_text(result.stdout + result.stderr)
        assert result.returncode == 0 and 'NEXUTERM_SUPERVISED_RECOVERY_OK' in result.stderr and 'CRITICAL' not in result.stderr, result.stderr
        # The intentionally interrupted fixture process is bounded to three seconds.
        time.sleep(3)
        print('PASS execution SIGKILL: durable partial output, unknown result and no replay on restart')


    with tempfile.TemporaryDirectory(prefix='nexuterm-execution-close-') as temporary:
        base = Path(temporary)
        config = base / 'config/nexuterm'
        config.mkdir(parents=True)
        (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[])))
        env = {k: v for k, v in os.environ.items() if not k.startswith('NEXUTERM_SMOKE') and k != 'NEXUTERM_SCREENSHOT'}
        env.update(HOME=temporary, XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / 'data'), XDG_STATE_HOME=str(base / 'state'), XDG_CACHE_HOME=str(base / 'cache'), SHELL='/bin/sh', GDK_BACKEND='x11', GSK_RENDERER='cairo', NEXUTERM_SMOKE_TEST='1', NEXUTERM_SMOKE_SUPERVISED='close')
        result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')], env=env, cwd=base, capture_output=True, text=True, timeout=25)
        (artifacts / 'supervised-close.log').write_text(result.stdout + result.stderr)
        assert result.returncode == 0 and 'NEXUTERM_SUPERVISED_CLOSE_REQUESTED' in result.stderr and 'panicked at' not in result.stderr and 'CRITICAL' not in result.stderr, result.stderr
        records = list((base / 'data/nexuterm/executions').glob('*.json'))
        assert len(records) == 1
        record = json.loads(records[0].read_text())
        assert record['status'] == 'Cancelled' and bytes(record['stdout']) == b'checkpoint', record
        assert not list(records[0].parent.glob('*.reserve'))
        print('PASS normal close: user confirms once, cancels running group, flushes final evidence before exit')


if __name__ == '__main__':
    main()
