#!/usr/bin/env python3
"""Intervention workflow in isolated GTK processes, including SIGKILL and no replay."""
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
ROOT = Path(__file__).resolve().parent.parent
ARTIFACTS = ROOT/'artifacts'

def environment(base,mode,theme='dark'):
    config=base/'config/nexuterm'
    config.mkdir(parents=True,exist_ok=True)
    if not (config/'sessions.json').exists():
        (config/'sessions.json').write_text(json.dumps(dict(version=2,profiles=[],preferences=dict(theme=theme))))
    env={k:v for k,v in os.environ.items() if not k.startswith('NEXUTERM_SMOKE') and k!='NEXUTERM_SCREENSHOT'}
    env.update(HOME=str(base),XDG_CONFIG_HOME=str(config.parent),XDG_DATA_HOME=str(base/'data'),XDG_STATE_HOME=str(base/'state'),XDG_CACHE_HOME=str(base/'cache'),SHELL='/bin/sh',GDK_BACKEND='x11',GSK_RENDERER='cairo',NEXUTERM_SMOKE_TEST='1',NEXUTERM_SMOKE_INTERVENTIONS=mode,NEXUTERM_INTERVENTION_SCREENSHOT=str(ARTIFACTS/f'interventions-{theme}.png'))
    return env

def command():
    return ['xvfb-run','-a','dbus-run-session','--',str(ROOT/'target/debug/nexuterm')]

def run(base,mode,marker,theme='dark'):
    started=time.monotonic()
    result=subprocess.run(command(),cwd=base,env=environment(base,mode,theme),capture_output=True,text=True,timeout=40)
    logs = result.stdout + result.stderr
    (ARTIFACTS/f'interventions-{theme}-{mode}.log').write_text(logs)
    assert result.returncode==0 and 'panicked at' not in logs and 'CRITICAL' not in logs,logs
    assert marker in logs,logs
    print(f'PASS intervention {mode}/{theme}: {time.monotonic()-started:.2f} s',flush=True)


def main():
    ARTIFACTS.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='nexuterm-intervention-light-') as temporary:
        base=Path(temporary)
        run(base,'run','NEXUTERM_SMOKE_INTERVENTION_OK','light')
        records=list((base/'data/nexuterm/interventions').glob('*.json'))
        assert len(records)==1
        value=json.loads(records[0].read_text())
        assert value['draft']['text']=='pendiente de recuperar'
        assert len(value['entries'])==5
        assert records[0].stat().st_mode & 0o777==0o600
        assert records[0].parent.stat().st_mode & 0o777==0o700
        assert (base/'reviewed.json').stat().st_mode & 0o777==0o600
        assert (ARTIFACTS/'interventions-light.png').is_file()
    with tempfile.TemporaryDirectory(prefix='nexuterm-intervention-crash-') as temporary:
        base=Path(temporary)
        log=ARTIFACTS/'interventions-crash.log'
        with log.open('w') as output:
            child=subprocess.Popen(command(),cwd=base,env=environment(base,'capture'),stdout=output,stderr=output)
            try:
                for _ in range(600):
                    lines=log.read_text().splitlines()
                    pid=next((int(line.split('=')[1]) for line in lines if line.startswith('NEXUTERM_INTERVENTION_CAPTURE_PID=')),None)
                    if pid is not None:
                        os.kill(pid,signal.SIGKILL)
                        break
                    assert child.poll() is None,log.read_text()
                    time.sleep(.05)
                else:
                    raise AssertionError(log.read_text())
                child.wait(timeout=10)
            finally:
                if child.poll() is None:
                    child.terminate()
                    child.wait(timeout=5)
        run(base,'restore','NEXUTERM_SMOKE_INTERVENTION_RESTORE_OK')
        assert (base/'counter-proof').read_text()=='once\n'
        assert (base/'crash-counter').read_text()=='once\n'
        assert not list((base/'data/nexuterm/interventions').glob('*.json'))
        assert not (base/'deleted-context-never').exists()
        time.sleep(3)
        print('PASS real SIGKILL: private drafts/approval/evidence retained, running result unknown, missing evidence explicit, deletion no resurrection, no replay',flush=True)
    with tempfile.TemporaryDirectory(prefix='nexuterm-intervention-close-') as temporary:
        base=Path(temporary)
        run(base,'close','NEXUTERM_INTERVENTION_CLOSE_REQUESTED')
        records=list((base/'data/nexuterm/interventions').glob('*.json'))
        assert len(records)==1
        assert json.loads(records[0].read_text())['draft']['text']=='último cambio antes de cerrar'
        print('PASS normal close: latest intervention draft flushed before exit',flush=True)

if __name__=='__main__':
    main()
