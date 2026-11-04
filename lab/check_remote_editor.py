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
            server = subprocess.Popen(["/usr/sbin/sshd", "-D", "-e", "-f", str(config)], stdout=log, stderr=log,
                                      start_new_session=True)
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
                                        "-i", str(base / "client"), "-l", getpass.getuser(), "-p", str(port), "127.0.0.1", "/bin/true"],
                                       capture_output=True, text=True, timeout=5)
                if probe.returncode:
                    log.seek(0)
                    raise RuntimeError(probe.stderr + log.read())
                cargo = shutil.which("cargo")
                subprocess.run([cargo, "test", "--locked", "--test", "remote_edit", "--", "--ignored", "--test-threads=1", "--nocapture"],
                               env=env, cwd=ROOT, check=True, timeout=120)
                print("PASS atomic SFTP: repeated save, inode replacement, metadata, recovery baseline, wrong destination, conflict and cleanup", flush=True)
                gui_ssh_tools(base, env, "dark")
                gui_ssh_tools(base, env, "light")
                gui_ssh_tools(base, env, "dark", close=True)
                gui(base, env, "dark")
                gui(base, env, "light")
                gui_compare(base, env, "dark")
                gui_compare(base, env, "light")
                gui_recovery(base, env, conflict=False)
                gui_recovery(base, env, conflict=True)
                gui_workspace_document(base, env)
            finally:
                server.terminate()
                server.wait(timeout=5)


def gui_ssh_tools(base, env, theme, close=False):
    mode = 'close' if close else theme
    folder = base / ('ssh-tools-' + mode)
    config = folder / 'config/nexuterm'
    config.mkdir(parents=True)
    profile = dict(id='remote-editor-fixture', name='SSH tunnel fixture', protocol='Ssh', host='127.0.0.1', port=int(env['NEXUTERM_REMOTE_FIXTURE_PORT']), username=env['NEXUTERM_REMOTE_FIXTURE_USER'], identity=str(base/'client'))
    (config/'sessions.json').write_text(json.dumps(dict(version=2, profiles=[profile], preferences=dict(theme=theme))))
    source = base/'ssh-import-config'
    source.write_text('Host imported-tool\n HostName example.invalid\n User demo\n Port 2222\nProxyCommand touch '+str(base/'never-imported')+'\nInclude never-read\nMatch exec touch '+str(base/'never-imported')+'\n')
    env = {k:v for k,v in env.items() if not k.startswith('NEXUTERM_SMOKE') and k != 'NEXUTERM_SCREENSHOT'}
    env.update(XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(folder/'data'), XDG_STATE_HOME=str(folder/'state'), XDG_CACHE_HOME=str(folder/'cache'), GSK_RENDERER='cairo', GDK_BACKEND='x11', NEXUTERM_SMOKE_TEST='1', NEXUTERM_SMOKE_SSH_TOOLS='close' if close else 'run', NEXUTERM_SSH_TOOLS_SCREENSHOT=str(ROOT/'artifacts'/('ssh-tools-'+mode+'.png')))
    for phase in (['close'] if close else ['run','restore']):
        env['NEXUTERM_SMOKE_SSH_TOOLS']=phase
        result=subprocess.run(['xvfb-run','-a','dbus-run-session','--',str(ROOT/'target/debug/nexuterm')],env=env,cwd=ROOT,capture_output=True,text=True,timeout=35)
        logs = result.stdout + result.stderr
        (ROOT/'artifacts'/('ssh-tools-'+mode+'-'+phase+'.log')).write_text(logs)
        assert result.returncode==0 and 'panicked at' not in logs and 'CRITICAL' not in logs,logs
        if phase!='close':
            marker='NEXUTERM_SMOKE_SSH_TOOLS_RESTORE_OK' if phase=='restore' else 'NEXUTERM_SMOKE_SSH_TOOLS_OK'
            assert marker in logs,logs
        else:
            port=int((base/'tunnel-close-port').read_text())
            with socket.socket() as probe:
                assert probe.connect_ex(('127.0.0.1',port)) != 0
        saved=json.loads((config/'sessions.json').read_text())
        assert len(saved['tunnels'])==1
        assert any(p['name']=='imported-tool' and p['ssh_ignore_config'] for p in saved['profiles'])
    assert not (base/'never-imported').exists()
    print(f'PASS SSH tools GTK {mode}: static import, explicit activation, real TCP traffic, cancellation, no replay and safe close',flush=True)


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
    logs = result.stdout + result.stderr
    (artifacts / f"remote-editor-code-{theme}.log").write_text(logs)
    assert result.returncode == 0 and "panicked at" not in logs, logs
    assert "NEXUTERM_SMOKE_REMOTE_SAVED_OK" in logs, logs
    assert "NEXUTERM_SMOKE_SYNTAX_OK" in logs, logs
    assert remote.read_text() == "def value():\n    return 42  # editado desde NexuTerm\n"
    print(f"PASS GTK {theme}: remote file activation, private download, syntax colours, edit, Ctrl+S and atomic SFTP replacement")


def gui_compare(base, env, theme):
    remote = base / 'remote' / ('compare-' + theme + '.py')
    remote.write_text('def value():\n    return 1\n')
    remote.chmod(0o640)
    config = base / ('compare-' + theme) / 'config/nexuterm'
    config.mkdir(parents=True)
    (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[], preferences=dict(theme=theme))))
    env = {k: v for k, v in env.items() if not k.startswith('NEXUTERM_SMOKE')}
    artifacts = ROOT / 'artifacts'
    env.update(XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / ('compare-' + theme) / 'data'), GSK_RENDERER='cairo', GDK_BACKEND='x11',
               NEXUTERM_SMOKE_TEST='1', NEXUTERM_SMOKE_REMOTE_EDITOR=str(remote), NEXUTERM_SMOKE_EDITOR_COMPARE='1',
               NEXUTERM_COMPARE_SCREENSHOT=str(artifacts / ('editor-compare-' + theme + '.png')))
    result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')], cwd=ROOT, env=env, capture_output=True, text=True, timeout=30)
    logs = result.stdout + result.stderr
    (artifacts / ('editor-compare-' + theme + '.log')).write_text(logs)
    assert result.returncode == 0 and 'panicked at' not in logs and 'CRITICAL' not in logs, logs
    assert 'NEXUTERM_SMOKE_EDITOR_COMPARE_OK' in logs, logs
    assert remote.read_text() == 'def value():\n    return 142\n'
    assert remote.stat().st_mode & 0o777 == 0o640
    print(f'PASS GTK compare {theme}: three versions, cancel, stale original, explicit result, durable base, guarded save and permissions', flush=True)


def gui_recovery(base, env, conflict):
    mode = "restore-ssh-conflict" if conflict else "restore-ssh-save"
    remote = base / "remote/recovered-ui.py"
    original = "def value():\n    return 1\n"
    changes = "def value():\n    return 84  # recuperado\n"
    remote.write_text("def value():\n    return 99  # externo\n" if conflict else original)
    destination = hashlib.sha256(json.dumps(["127.0.0.1", int(env["NEXUTERM_REMOTE_FIXTURE_PORT"]), env["NEXUTERM_REMOTE_FIXTURE_USER"]], separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()
    data = base / mode / "data"
    directory = data / "nexuterm/recovery"
    directory.mkdir(parents=True, mode=0o700)
    identifier = str(uuid.uuid4())
    draft = dict(version=1, id=identifier, source=dict(kind="Remote", profile_id="remote-editor-fixture", host="127.0.0.1", destination=destination, path=str(remote), fingerprint=hashlib.sha256(original.encode()).hexdigest()), text=changes)
    path = directory / (identifier + ".json")
    path.write_text(json.dumps(draft)); path.chmod(0o600)
    config = base / mode / "config/nexuterm"
    config.mkdir(parents=True)
    (config / "sessions.json").write_text(json.dumps(dict(version=2, profiles=[])))
    env = {k: v for k, v in env.items() if not k.startswith("NEXUTERM_SMOKE")}
    env.update(XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(data), GSK_RENDERER="cairo", GDK_BACKEND="x11", NEXUTERM_SMOKE_TEST="1", NEXUTERM_SMOKE_REMOTE_REBIND="1", NEXUTERM_SMOKE_RECOVERY=mode)
    artifacts = ROOT / "artifacts"
    env["NEXUTERM_SCREENSHOT"] = str(artifacts / (mode + ".png"))
    result = subprocess.run(["xvfb-run", "-a", "dbus-run-session", "--", str(ROOT / "target/debug/nexuterm")], cwd=base, env=env, capture_output=True, text=True, timeout=25)
    logs = result.stdout + result.stderr
    (artifacts / (mode + ".log")).write_text(logs)
    assert result.returncode == 0 and "CRITICAL" not in logs, logs
    marker = "NEXUTERM_SMOKE_RECOVERY_REBIND_" + ("CONFLICT" if conflict else "SAVED") + "_OK"
    assert marker in logs, logs
    expected = "def value():\n    return 99  # externo\n" if conflict else changes
    assert remote.read_text() == expected
    stored = json.loads(path.read_text())
    if not conflict:
        assert stored["source"]["fingerprint"] == hashlib.sha256(changes.encode()).hexdigest()
        assert stored["text"] == changes + "# siguiente edición\n"
    else:
        assert "NEXUTERM_SMOKE_LEGACY_COMPARISON_OK" in logs
        assert stored["base_text"] == expected
        assert stored["text"] == changes
    print("PASS GTK " + mode + ": explicit authenticated binding, guarded save and durable baseline")


def gui_workspace_document(base, env):
    remote = base / "remote/workspace-clean.txt"
    remote.write_text("Documento SSH actualizado\n")
    destination = hashlib.sha256(json.dumps(["127.0.0.1", int(env["NEXUTERM_REMOTE_FIXTURE_PORT"]), env["NEXUTERM_REMOTE_FIXTURE_USER"]], separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()
    source = dict(kind="Remote", profile_id="remote-editor-fixture", host="127.0.0.1", destination=destination, path=str(remote), fingerprint=hashlib.sha256(b"older contents").hexdigest())
    data = base / "workspace-document/data"
    directory = data / "nexuterm/workspaces"
    directory.mkdir(parents=True, mode=0o700)
    layout = dict(kind="Document", id=str(uuid.uuid4()), source=source, had_draft=False)
    workspace = dict(id="ssh-workspace", name="Documento SSH", notes="", tabs=[dict(title="Documento SSH", layout=layout)], selected=0, directory=None, sidebar_visible=True)
    path = directory / "workspaces.json"
    path.write_text(json.dumps(dict(version=1, workspaces=[workspace]))); path.chmod(0o600)
    config = base / "workspace-document/config/nexuterm"
    config.mkdir(parents=True)
    (config / "sessions.json").write_text(json.dumps(dict(version=2, profiles=[])))
    env = {k: v for k, v in env.items() if not k.startswith("NEXUTERM_SMOKE")}
    env.update(XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(data), GSK_RENDERER="cairo", GDK_BACKEND="x11", NEXUTERM_SMOKE_TEST="1", NEXUTERM_SMOKE_REMOTE_REBIND="1", NEXUTERM_SMOKE_WORKSPACE_DOCUMENTS="remote-clean", NEXUTERM_DOCUMENT_FIXTURE=str(base))
    artifacts = ROOT / "artifacts"
    env["NEXUTERM_SCREENSHOT"] = str(artifacts / "workspace-document-ssh.png")
    result = subprocess.run(["xvfb-run", "-a", "dbus-run-session", "--", str(ROOT / "target/debug/nexuterm")], cwd=base, env=env, capture_output=True, text=True, timeout=25)
    logs = result.stdout + result.stderr
    (artifacts / "workspace-document-ssh.log").write_text(logs)
    assert result.returncode == 0 and "CRITICAL" not in logs, logs
    assert "NEXUTERM_SMOKE_DOCUMENTS_REMOTE_CLEAN_OK" in logs, logs
    assert remote.read_text() == "Documento SSH actualizado\n"
    assert not list((data / "nexuterm/recovery").glob("*.json"))
    print("PASS GTK workspace SSH document: explicit existing-session open, current original, stable position and no duplicate")


if __name__ == "__main__":
    main()
