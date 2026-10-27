#!/usr/bin/env python3
"""Crash/restore editor drafts using temporary XDG data and no remote services."""
import copy
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
CONTENTS = "Borrador recuperable con acentos áé\nÚltima edición sin guardar\n"


def run(base, mode, kill=False):
    config = base / "config/nexuterm"
    config.mkdir(parents=True, exist_ok=True)
    (config / "sessions.json").write_text(json.dumps(dict(version=2, profiles=[])))
    env = {k: v for k, v in os.environ.items()
           if not k.startswith("NEXUTERM_SMOKE") and k != "NEXUTERM_SCREENSHOT"}
    env.update(HOME=str(base), XDG_CONFIG_HOME=str(config.parent),
               XDG_DATA_HOME=str(base / "data"), XDG_CACHE_HOME=str(base / "cache"),
               XDG_STATE_HOME=str(base / "state"), GSK_RENDERER="cairo", GDK_BACKEND="x11",
               NEXUTERM_SMOKE_TEST="1", NEXUTERM_SMOKE_RECOVERY=mode,
               NEXUTERM_RECOVERY_FILE=str(base / "original.txt"),
               NEXUTERM_RECOVERY_COPY=str(base / (mode + "-copy.txt")),
               NEXUTERM_SCREENSHOT=str(ARTIFACTS / ("recovery-" + mode + ".png")))
    log_path = ARTIFACTS / ("recovery-" + mode + ".log")
    with log_path.open("w+") as log:
        process = subprocess.Popen(["xvfb-run", "-a", "dbus-run-session", "--",
                                    str(ROOT / "target/debug/nexuterm")], cwd=base, env=env,
                                   stdout=log, stderr=log, start_new_session=True)
        try:
            if kill:
                deadline = time.monotonic() + 12
                while time.monotonic() < deadline:
                    if "NEXUTERM_SMOKE_RECOVERY_CAPTURE_OK" in log_path.read_text():
                        break
                    if process.poll() is not None:
                        raise AssertionError(log_path.read_text())
                    time.sleep(.05)
                else:
                    raise AssertionError("Autosave did not finish: " + log_path.read_text())
                pid_line = next(line for line in log_path.read_text().splitlines()
                                if line.startswith("NEXUTERM_RECOVERY_PID="))
                os.kill(int(pid_line.split("=", 1)[1]), signal.SIGKILL)
                assert process.wait(timeout=5) != 0
            else:
                assert process.wait(timeout=15) == 0, log_path.read_text()
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=5)
    output = log_path.read_text()
    assert "panicked at" not in output and "CRITICAL" not in output, output
    marker = {"restore-save": "SAVE", "restore-conflict": "RESTORE",
              "restore-missing": "RESTORE", "restore-remote": "RESTORE",
              "discard": "DISCARD", "capture": "CAPTURE", "fail-close": "FAIL_CLOSE"}.get(mode)
    if marker:
        assert "NEXUTERM_SMOKE_RECOVERY_" + marker + "_OK" in output, output
    return base / "data/nexuterm/recovery"


def main():
    ARTIFACTS.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="nexuterm-recovery-") as temporary:
        base = Path(temporary)
        original = base / "original.txt"
        original.write_text("Original intacto\n")
        directory = run(base, "capture", kill=True)
        records = list(directory.glob("*.json"))
        assert len(records) == 1
        path = records[0]
        snapshot = json.loads(path.read_text())
        assert snapshot["text"] == CONTENTS
        assert original.read_text() == "Original intacto\n"
        assert directory.stat().st_mode & 0o777 == 0o700
        assert path.stat().st_mode & 0o777 == 0o600
        print("PASS crash: actual SIGKILL after periodic autosave; original untouched; private draft")

        run(base, "restore-save")
        assert original.read_text() == CONTENTS and not list(directory.glob("*.json"))
        print("PASS recovery: restored modified buffer, etag save, draft removed after success")

        path.write_text(json.dumps(snapshot))
        original.write_text("Cambio externo que debe conservarse\n")
        run(base, "restore-conflict")
        assert original.read_text() == "Cambio externo que debe conservarse\n"
        assert (base / "restore-conflict-copy.txt").read_text() == CONTENTS
        assert list(directory.glob("*.json"))
        print("PASS conflict: original save disabled; comparison visible; exported copy; draft retained")

        original.unlink()
        run(base, "restore-missing")
        assert not original.exists()
        assert (base / "restore-missing-copy.txt").read_text() == CONTENTS
        print("PASS missing original: text recovered without recreating the original")

        path.unlink()
        remote = copy.deepcopy(snapshot)
        remote["id"] = str(uuid.uuid4())
        remote["source"] = dict(kind="Remote", profile_id="deleted-profile", host="fixture.invalid",
                                destination="0" * 64, path="/obsolete-temporary/location.py")
        remote_path = directory / (remote["id"] + ".json")
        remote_path.write_text(json.dumps(remote))
        run(base, "restore-remote")
        assert (base / "restore-remote-copy.txt").read_text() == CONTENTS
        assert remote_path.exists()
        print("PASS offline remote: recovered without server, credentials or temporary working copy")

        run(base, "discard")
        assert not list(directory.glob("*.json"))
        print("PASS explicit discard: confirmed through GTK and removed durably")

    with tempfile.TemporaryDirectory(prefix="nexuterm-recovery-close-") as temporary:
        base = Path(temporary)
        (base / "original.txt").write_text("Original\n")
        directory = run(base, "close")
        records = list(directory.glob("*.json"))
        assert len(records) == 1 and json.loads(records[0].read_text())["text"] == CONTENTS
        assert (base / "original.txt").read_text() == "Original\n"
        print("PASS normal close: latest change flushed before periodic autosave, original untouched")

    with tempfile.TemporaryDirectory(prefix="nexuterm-recovery-error-") as temporary:
        base = Path(temporary)
        (base / "original.txt").write_text("Original\n")
        run(base, "fail-close")
        assert (base / "original.txt").read_text() == "Original\n"
        print("PASS failed autosave: normal close blocked and modified editor kept open")


if __name__ == "__main__":
    main()
