#!/usr/bin/env python3
"""Workspace/editor integration across processes, with private synthetic XDG data."""
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import uuid

ROOT = Path(__file__).resolve().parent.parent
ARTIFACTS = ROOT / "artifacts"


def run(base, mode, crash=False):
    config = base / "config/nexuterm"
    config.mkdir(parents=True, exist_ok=True)
    (config / "sessions.json").write_text(json.dumps(dict(version=2, profiles=[])))
    env = {k: v for k, v in os.environ.items() if not k.startswith("NEXUTERM_SMOKE") and k != "NEXUTERM_SCREENSHOT"}
    env.update(HOME=str(base), XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / "data"),
               XDG_CACHE_HOME=str(base / "cache"), XDG_STATE_HOME=str(base / "state"),
               NEXUTERM_SMOKE_TEST="1", NEXUTERM_SMOKE_WORKSPACE_DOCUMENTS=mode,
               NEXUTERM_DOCUMENT_FIXTURE=str(base), GSK_RENDERER="cairo", GDK_BACKEND="x11", SHELL="/bin/sh",
               NEXUTERM_SCREENSHOT=str(ARTIFACTS / ("workspace-documents-" + mode + ".png")))
    log_path = ARTIFACTS / ("workspace-documents-" + mode + ".log")
    with log_path.open("w+") as log:
        process = subprocess.Popen(["xvfb-run", "-a", "dbus-run-session", "--", str(ROOT / "target/debug/nexuterm")],
                                   cwd=base, env=env, stdout=log, stderr=log, start_new_session=True)
        try:
            if crash:
                deadline = time.monotonic() + 12
                while time.monotonic() < deadline:
                    text = log_path.read_text()
                    if "NEXUTERM_SMOKE_DOCUMENTS_SAVE_OK" in text:
                        line = next(line for line in text.splitlines() if line.startswith("NEXUTERM_DOCUMENTS_PID="))
                        os.kill(int(line.split("=", 1)[1]), signal.SIGKILL)
                        assert process.wait(timeout=5) != 0
                        break
                    if process.poll() is not None:
                        raise AssertionError(text)
                    time.sleep(.05)
                else:
                    raise AssertionError("Workspace save did not finish: " + log_path.read_text())
            else:
                assert process.wait(timeout=15) == 0, log_path.read_text()
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=5)
    output = log_path.read_text()
    assert "CRITICAL" not in output and "panicked at" not in output, output
    marker = "SAVE" if mode == "save" else "SAVE_FAILURE" if mode == "save-failure" else "RESTORE"
    assert "NEXUTERM_SMOKE_DOCUMENTS_" + marker + "_OK" in output, output


def main():
    ARTIFACTS.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="nexuterm-workspace-documents-") as temporary:
        base = Path(temporary)
        (base / "clean.txt").write_text("Original limpio\n")
        (base / "dirty.txt").write_text("Original sin editar\n")
        directory = base / "data/nexuterm/recovery"
        directory.mkdir(parents=True, mode=0o700)
        identifier = str(uuid.uuid4())
        draft = dict(version=1, id=identifier, source=dict(kind="Remote", profile_id="offline", host="fixture.invalid", destination="0" * 64, path="/remote/offline.py"), text="Borrador SSH sin conexión\n")
        seed = json.dumps(draft)
        (base / "remote-seed.json").write_text(seed)
        (directory / (identifier + ".json")).write_text(seed)
        run(base, "save", crash=True)
        workspace_path = base / "data/nexuterm/workspaces/workspaces.json"
        saved = json.loads(workspace_path.read_text())["workspaces"][0]
        references = saved["tabs"][1:]
        assert [tab["layout"]["had_draft"] for tab in references] == [False, True, True]
        records = {path.name: path.read_bytes() for path in directory.glob("*.json")}
        assert len(records) == 2
        assert "Cambios pendientes" not in workspace_path.read_text()
        assert (base / "dirty.txt").read_text() == "Original sin editar\n"
        print("PASS workspace save + SIGKILL: ordered editor identities, durable draft references, no copied text or original writes")
        dirty_id = references[1]["layout"]["id"]
        remote_id = references[2]["layout"]["id"]
        for mode in ("restore", "missing-draft", "missing-original", "corrupt-draft", "corrupt-local"):
            for filename, data in records.items():
                (directory / filename).write_bytes(data)
            (base / "clean.txt").write_text("Original limpio actualizado\n")
            (base / "dirty.txt").write_text("Cambio externo\n")
            if mode == "missing-draft":
                (directory / (dirty_id + ".json")).unlink()
            if mode == "missing-original":
                (base / "dirty.txt").unlink()
            if mode == "corrupt-draft":
                (directory / (remote_id + ".json")).write_bytes(b"corrupt")
            if mode == "corrupt-local":
                (directory / (dirty_id + ".json")).write_bytes(b"corrupt")
            run(base, mode)
            assert (base / "clean.txt").read_text() == "Original limpio actualizado\n"
            if mode == "missing-original":
                assert not (base / "dirty.txt").exists()
            else:
                assert (base / "dirty.txt").read_text() == "Cambio externo\n"
            if mode == "corrupt-local":
                assert (directory / (dirty_id + ".json")).read_bytes() == b"corrupt"
            print("PASS workspace " + mode + ": order, selected tab, preserved contents and no automatic connection")
    with tempfile.TemporaryDirectory(prefix="nexuterm-workspace-save-error-") as temporary:
        base = Path(temporary)
        (base / "dirty.txt").write_text("Original\n")
        data = base / "data/nexuterm"
        directory = data / "workspaces"
        directory.mkdir(parents=True)
        (data / "recovery").write_text("blocked")
        original = dict(version=1, workspaces=[dict(id="previous", name="Anterior", notes="intacto", tabs=[], selected=0, directory=None, sidebar_visible=True)])
        path = directory / "workspaces.json"
        path.write_text(json.dumps(original))
        run(base, "save-failure")
        assert json.loads(path.read_text()) == original
        assert (base / "dirty.txt").read_text() == "Original\n"
        print("PASS workspace save failure: previous workspace intact and document stays modified")


if __name__ == "__main__":
    main()
