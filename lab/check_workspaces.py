#!/usr/bin/env python3
"""Persist and restore a workspace across isolated GTK processes; no remote hosts."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
ARTIFACTS = ROOT / "artifacts"


def main():
    ARTIFACTS.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="nexuterm-workspaces-") as temporary:
        directory = Path(temporary)
        config = directory / "config/nexuterm"
        config.mkdir(parents=True)
        (config / "sessions.json").write_text(json.dumps(dict(version=2, profiles=[])))
        env = {k: v for k, v in os.environ.items()
               if not k.startswith("NEXUTERM_SMOKE") and k != "NEXUTERM_SCREENSHOT"}
        env.update(HOME=temporary, XDG_CONFIG_HOME=str(config.parent),
                   XDG_DATA_HOME=str(directory / "data"), XDG_STATE_HOME=str(directory / "state"),
                   XDG_CACHE_HOME=str(directory / "cache"), NEXUTERM_SMOKE_TEST="1",
                   GSK_RENDERER="cairo", GDK_BACKEND="x11", SHELL="/bin/sh")
        for mode in ("save", "restore", "missing-folder", "missing-profile"):
            if mode.startswith("missing-"):
                path = directory / "data/nexuterm/workspaces/workspaces.json"
                saved = json.loads(path.read_text())
                layout = saved["workspaces"][0]["tabs"][0]["layout"]
                while layout["kind"] == "Split":
                    layout = layout["first"]
                layout["target"] = (dict(kind="Local", directory=str(directory / "absent"))
                                    if mode == "missing-folder" else dict(kind="Connection", profile_id="deleted"))
                path.write_text(json.dumps(saved))
            env.update(NEXUTERM_WORKSPACE_SCREENSHOT=str(ARTIFACTS / "workspaces-dialog.png"),
                       NEXUTERM_SMOKE_WORKSPACES=mode,
                       NEXUTERM_SCREENSHOT=str(ARTIFACTS / f"workspaces-{mode}.png"))
            result = subprocess.run(["xvfb-run", "-a", "dbus-run-session", "--",
                                     str(ROOT / "target/debug/nexuterm")],
                                    cwd=directory, env=env, capture_output=True, text=True, timeout=20)
            (ARTIFACTS / f"workspaces-{mode}.log").write_text(result.stdout + result.stderr)
            assert result.returncode == 0, result.stderr
            assert "CRITICAL" not in result.stderr, result.stderr
            marker = "NEXUTERM_SMOKE_WORKSPACES_" + mode.upper().replace("-", "_") + "_OK"
            assert marker in result.stderr, result.stderr
            print(f"PASS workspace {mode}: isolated GTK process")
        path = directory / "data/nexuterm/workspaces/workspaces.json"
        saved = json.loads(path.read_text())
        assert saved["workspaces"][0]["name"] == "Cliente renombrado"
        assert path.stat().st_mode & 0o777 == 0o600
        assert path.parent.stat().st_mode & 0o777 == 0o700
        print("PASS workspace persistence: notes, rename, private permissions, split layout and explicit start")


if __name__ == "__main__":
    main()
