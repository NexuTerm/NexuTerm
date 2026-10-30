#!/usr/bin/env python3
"""Verify editor and browser view restoration after asynchronous loading without contacting remote hosts."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def main():
    with tempfile.TemporaryDirectory(prefix="nexuterm-workspace-views-") as temporary:
        directory = Path(temporary)
        config = directory / "config/nexuterm"
        config.mkdir(parents=True)
        (config / "sessions.json").write_text(json.dumps(dict(version=2, profiles=[])))
        contents=directory / "views"
        contents.mkdir()
        (contents / "initial").mkdir()
        (contents / "texto.txt").write_text("Texto áéíóú de varias líneas para comprobar el cursor.\n" * 1000)
        (contents / ".selección á.txt").write_text("fixture")
        for i in range(140):
            (contents / f"archivo-{i:03}.txt").write_text("fixture")
        env = {k: v for k, v in os.environ.items()
               if not k.startswith("NEXUTERM_SMOKE") and k != "NEXUTERM_SCREENSHOT"}
        env.update(HOME=temporary, XDG_CONFIG_HOME=str(config.parent),
                   XDG_DATA_HOME=str(directory / "data"), XDG_STATE_HOME=str(directory / "state"),
                   XDG_CACHE_HOME=str(directory / "cache"), NEXUTERM_SMOKE_TEST="1", NEXUTERM_VIEWS_ROOT=str(contents),
                   NEXUTERM_SMOKE_EDITOR_BROWSER_VIEWS="1", NEXUTERM_SCREENSHOT=str(ROOT / "artifacts/workspace-views.png"), GSK_RENDERER="cairo",
                   GDK_BACKEND="x11", SHELL="/bin/sh")
        result = subprocess.run(["xvfb-run", "-a", "dbus-run-session", "--",
                                 str(ROOT / "target/debug/nexuterm")],
                                cwd=directory, env=env, capture_output=True, text=True, timeout=20)
        artifacts = ROOT / "artifacts"
        artifacts.mkdir(exist_ok=True)
        (artifacts / "workspace-views.log").write_text(result.stdout + result.stderr)
        assert result.returncode == 0, result.stderr
        assert "CRITICAL" not in result.stderr, result.stderr
        assert "NEXUTERM_SMOKE_EDITOR_BROWSER_VIEWS_OK" in result.stderr, result.stderr
        print("PASS editor/browser restoration: cursor, selection, reading position, hidden files, load races and missing selection")


if __name__ == "__main__":
    main()
