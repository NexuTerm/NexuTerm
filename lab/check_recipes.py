#!/usr/bin/env python3
"""GTK read-only recipes: real df, synthetic service/journal and isolated XDG data."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
ROOT = Path(__file__).resolve().parent.parent


def main():
    artifacts = ROOT / 'artifacts'
    artifacts.mkdir(exist_ok=True)
    for theme in ('dark', 'light'):
        with tempfile.TemporaryDirectory(prefix='nexuterm-recipes-') as temporary:
            base = Path(temporary)
            config = base / 'config/nexuterm'
            config.mkdir(parents=True)
            (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[], preferences=dict(theme=theme))))
            binary = base / 'bin'
            binary.mkdir()
            tools = {
                'systemctl': 'case "$1" in show) printf "LoadState=loaded\\nActiveState=active\\nSubState=running\\nResult=success\\n";; is-active) [ ! -e "$HOME/service-inactive" ] || exit 3;; *) exit 9;; esac',
                'journalctl': '[ "$3" = "--lines=12" ] && [ "$4" = "--since=-5 minutes" ] && [ "$5" = "--unit=demo.service" ] || exit 9\nprintf "fixture journal\\n"',
            }
            for name, text in tools.items():
                path = binary / name
                path.write_text('#!/bin/sh\n' + text + '\n')
                path.chmod(0o700)
            env = {k:v for k,v in os.environ.items() if not k.startswith('NEXUTERM_SMOKE') and k != 'NEXUTERM_SCREENSHOT'}
            env.update(HOME=temporary, XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / 'data'), XDG_STATE_HOME=str(base / 'state'), XDG_CACHE_HOME=str(base / 'cache'), SHELL='/bin/sh', PATH=str(binary)+os.pathsep+env['PATH'], GDK_BACKEND='x11', GSK_RENDERER='cairo', NEXUTERM_SMOKE_TEST='1', NEXUTERM_SMOKE_SUPERVISED='recipes', NEXUTERM_SUPERVISED_SCREENSHOT=str(artifacts / f'recipes-{theme}.png'))
            result = subprocess.run(['xvfb-run','-a','dbus-run-session','--',str(ROOT/'target/debug/nexuterm')],env=env,cwd=base,capture_output=True,text=True,timeout=25)
            logs = result.stdout + result.stderr
            (artifacts / f'recipes-{theme}.log').write_text(logs)
            assert result.returncode == 0 and 'NEXUTERM_SMOKE_RECIPES_OK' in logs, logs
            assert 'CRITICAL' not in logs and 'panicked at' not in logs, logs
            records=list((base/'data/nexuterm/executions').glob('*.json'))
            assert len(records)==4
            assert sorted(json.loads(p.read_text())['exit_code'] for p in records) in ([0,0,0,3],[0,0,2,3])
            assert all(p.stat().st_mode & 0o777 == 0o600 for p in records)
            print(f'PASS recipes {theme}: reject/approve, literal disk path, active/inactive service, bounded journal, validation, private history and manual draft preserved')


if __name__=='__main__':
    main()
