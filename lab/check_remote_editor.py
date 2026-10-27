#!/usr/bin/env python3
"""Atomic SFTP saves and draft recovery against a disposable loopback sshd."""
import getpass
import hashlib
import uuid
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parent.parent


def main():
    with tempfile.TemporaryDirectory(prefix=".nx-edit-test-", dir=Path.home()) as directory:
        base = Path(directory)
        (base / "remote").mkdir()
        for name in ("client", "host"):
            subprocess.run(["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-f", str(base / name)], check=True)
        (base / "authorized_keys").write_bytes((base / "client.pub").read_bytes())
        (base / "authorized_keys").chmod(0o600)
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        (base / "known_hosts").write_text(f"[127.0.0.1]:{port} " + (base / "host.pub").read_text())
        config = base / "sshd_config"
        config.write_text(f"Port {port}\nListenAddress 127.0.0.1\nHostKey {base / 'host'}\nPidFile {base / 'sshd.pid'}\nAuthorizedKeysFile {base / 'authorized_keys'}\nPasswordAuthentication no\nKbdInteractiveAuthentication no\nUsePAM no\nStrictModes yes\nAllowUsers {getpass.getuser()}\nSubsystem sftp internal-sftp\nLogLevel VERBOSE\n")
        env = dict(os.environ, NEXUTERM_REMOTE_FIXTURE_ROOT=directory,
                   NEXUTERM_REMOTE_FIXTURE_PORT=str(port), NEXUTERM_REMOTE_FIXTURE_USER=getpass.getuser())
        with (base / "sshd.log").open("w+") as log:
            server = subprocess.Popen(["/usr/sbin/sshd", "-D", "-e", "-f", str(config)], stdout=log, stderr=log)
            try:
                for _ in range(40):
                    if server.poll() is not None:
                        log.seek(0)
                        raise RuntimeError(log.read())
                    try:
                        with socket.create_connection(("127.0.0.1", port), timeout=.1):
                            break
                    except OSError:
                        time.sleep(.05)
                probe = subprocess.run(["ssh", "-F", "/dev/null", "-o", "BatchMode=yes", "-o", "IdentitiesOnly=yes",
                                        "-o", "StrictHostKeyChecking=yes", "-o", f"UserKnownHostsFile={base / 'known_hosts'}",
                                        "-i", str(base / "client"), "-p", str(port), "127.0.0.1", "/bin/true"],
                                       capture_output=True, text=True, timeout=5)
                if probe.returncode:
                    log.seek(0)
                    raise RuntimeError(probe.stderr + log.read())
                cargo = shutil.which("cargo")
                subprocess.run([cargo, "test", "--locked", "--test", "remote_edit", "--", "--ignored", "--test-threads=1"],
                               env=env, cwd=ROOT, check=True, timeout=120)
                print("PASS atomic SFTP: repeated save, inode replacement, metadata, recovery baseline, wrong destination, conflict and cleanup", flush=True)
                gui(base, env, "dark")
                gui(base, env, "light")
            finally:
                server.terminate()
                server.wait(timeout=5)


def gui(base, env, theme):
    remote = base / "remote" / "gui example.py"
    remote.write_text("def value():\n    return 1\n")
    config = base / ("config-" + theme) / "nexuterm"
    config.mkdir(parents=True)
    (config / "sessions.json").write_text(json.dumps(dict(version=2, profiles=[], preferences=dict(theme=theme))))
    env = {k: v for k, v in env.items() if not k.startswith("NEXUTERM_SMOKE")}
    env.update(XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / "data"), GSK_RENDERER="cairo", GDK_BACKEND="x11",
               NEXUTERM_SMOKE_TEST="1", NEXUTERM_SMOKE_REMOTE_EDITOR=str(remote), NEXUTERM_SMOKE_SYNTAX="1")
    artifacts = ROOT / "artifacts"
    artifacts.mkdir(exist_ok=True)
    env["NEXUTERM_SCREENSHOT"] = str(artifacts / f"remote-editor-code-{theme}.png")
    result = subprocess.run(["xvfb-run", "-a", "dbus-run-session", "--", str(ROOT / "target/debug/nexuterm")],
                            cwd=ROOT, env=env, capture_output=True, text=True, timeout=25)
    (artifacts / f"remote-editor-code-{theme}.log").write_text(result.stdout + result.stderr)
    assert result.returncode == 0 and "panicked at" not in result.stderr, result.stderr
    assert "NEXUTERM_SMOKE_REMOTE_SAVED_OK" in result.stderr, result.stderr
    assert "NEXUTERM_SMOKE_SYNTAX_OK" in result.stderr, result.stderr
    assert remote.read_text() == "def value():\n    return 42  # editado desde NexuTerm\n"
    print(f"PASS GTK {theme}: remote file activation, private download, syntax colours, edit, Ctrl+S and atomic SFTP replacement")


if __name__ == "__main__":
    main()
