#!/usr/bin/env python3
"""Run a test in its own Secret Service; call only inside dbus-run-session."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

with tempfile.TemporaryDirectory(prefix='nexuterm-keyring-') as directory:
    root = Path(directory)
    env = os.environ.copy()
    env.pop("GNOME_KEYRING_PID", None)
    env.update(HOME=str(root), XDG_DATA_HOME=str(root / 'data'),
               XDG_RUNTIME_DIR=str(root), GNOME_KEYRING_CONTROL=str(root / 'control'))
    daemon = subprocess.Popen(['gnome-keyring-daemon', '--foreground', '--unlock',
                               '--components=secrets', '--control-directory', str(root / 'control')],
                              env=env, stdin=subprocess.PIPE,
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        daemon.stdin.write(b'fictitious-keyring-unlock\n')
        daemon.stdin.close()
        result = subprocess.run(sys.argv[1:], env=env, timeout=15)
        raise SystemExit(result.returncode)
    finally:
        daemon.terminate()
        try:
            daemon.wait(timeout=3)
        except subprocess.TimeoutExpired:
            daemon.kill()
            daemon.wait()
