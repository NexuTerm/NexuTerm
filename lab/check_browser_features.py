#!/usr/bin/env python3
"""Permissions, hidden files, Preferences and safe creation through the real GTK controls."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def check(theme):
    with tempfile.TemporaryDirectory(prefix="nexuterm-browser-") as directory:
        base = Path(directory)
        local = base / "local"
        local.mkdir()
        (local / "nested").mkdir(mode=0o750)
        (local / "nested").chmod(0o750)
        (local / ".hidden-file").write_text("hidden")
        (local / ".hidden-file").chmod(0o640)
        (local / ".hidden-dir").mkdir()
        (local / "existing.txt").write_text("keep original")
        for index in range(80):
            (local / f"visible-{index:02}.txt").touch()
        config = base / "config" / "nexuterm"
        config.mkdir(parents=True)
        path = config / "sessions.json"
        path.write_text(json.dumps(dict(version=2, profiles=[], preferences=dict(theme=theme))))
        artifacts = ROOT / "artifacts"
        artifacts.mkdir(exist_ok=True)
        env = {k: v for k, v in os.environ.items() if not k.startswith("NEXUTERM_SMOKE")}
        env.update(XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / "data"), GSK_RENDERER="cairo",
                   NEXUTERM_SMOKE_TEST="1", NEXUTERM_SMOKE_BROWSER_FEATURES="1",
                   NEXUTERM_SMOKE_FILE_BROWSER_LOCAL=str(local),
                   NEXUTERM_SCREENSHOT=str(artifacts / f"browser-features-{theme}.png"),
                   NEXUTERM_BROWSER_PREFERENCES_SCREENSHOT=str(artifacts / f"browser-preferences-{theme}.png"))
        result = subprocess.run(["xvfb-run", "-a", "-s", "-screen 0 1920x1080x24", "dbus-run-session", "--",
                                 str(ROOT / "target/debug/nexuterm")], cwd=local, env=env,
                                capture_output=True, text=True, timeout=25)
        (artifacts / f"browser-features-{theme}.log").write_text(result.stdout + result.stderr)
        assert result.returncode == 0 and "panicked at" not in result.stderr, result.stderr
        for marker in ("BROWSER_HIDDEN_OK", "BROWSER_PREFERENCES_OK", "BROWSER_CREATE_AND_SCROLL_OK"):
            assert "NEXUTERM_SMOKE_" + marker in result.stderr, result.stderr
        assert (local / "created-dir").is_dir()
        assert (local / "created.txt").read_bytes() == b""
        assert (local / "existing.txt").read_text() == "keep original"
        assert not (base / "outside").exists()
        saved = json.loads(path.read_text())["preferences"]
        assert not saved["browser_show_type"] and not saved["browser_show_size"]
        assert saved["browser_show_permissions"] and not saved["browser_show_hidden"]
        print(f"PASS browser {theme}: permissions, Ctrl+H, toggle, preferences, creation, conflicts, scroll")


if __name__ == "__main__":
    check("dark")
    check("light")
