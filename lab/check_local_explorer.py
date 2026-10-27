#!/usr/bin/env python3
"""Local explorer/editor smoke test using isolated files and configuration."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import sys

ROOT = Path(__file__).resolve().parent.parent


def check(theme, numbers=True, external=False):
    with tempfile.TemporaryDirectory(prefix="nexuterm-local-explorer-") as directory:
        base = Path(directory)
        local = base / "local"
        nested = local / "nested"
        nested.mkdir(parents=True)
        (nested / "sample con á.txt").write_text("primera línea\nsegunda línea\n", encoding="utf-8")
        config = base / "config" / "nexuterm"
        config.mkdir(parents=True)
        (config / "sessions.json").write_text(json.dumps(dict(version=2, profiles=[], preferences=dict(theme=theme, editor_line_numbers=numbers, files_use_system_app=external))))
        env = {k: v for k, v in os.environ.items() if not k.startswith("NEXUTERM_SMOKE") and k != "NEXUTERM_SCREENSHOT"}
        if external:
            applications = base / "data" / "applications"
            applications.mkdir(parents=True)
            handler = base / "external-handler.py"
            handler.write_text("import json, pathlib, sys\npathlib.Path(sys.argv[1]).write_text(json.dumps(sys.argv[2:]))\n")
            (applications / "nexuterm-test-editor.desktop").write_text(
                f'[Desktop Entry]\nType=Application\nName=NexuTerm test editor\nExec="{sys.executable}" "{handler}" "{base / "opened.json"}" %f\nMimeType=text/plain;\nNoDisplay=true\n')
            (config.parent / "mimeapps.list").write_text("[Default Applications]\ntext/plain=nexuterm-test-editor.desktop;\n")
        tag = f"local-explorer-{theme}-numbers-{numbers}-external-{external}"
        artifacts = ROOT / "artifacts"
        artifacts.mkdir(exist_ok=True)
        env.update(XDG_CONFIG_HOME=str(config.parent), XDG_DATA_HOME=str(base / "data"),
                   NEXUTERM_SMOKE_TEST="1", NEXUTERM_SMOKE_LOCAL_EXPLORER="1",
                   NEXUTERM_SMOKE_FILE_BROWSER_LOCAL=str(local), GSK_RENDERER="cairo", GDK_BACKEND="x11",
                   NEXUTERM_SCREENSHOT=str(artifacts / f"{tag}.png"))
        if not numbers:
            env["NEXUTERM_SMOKE_NO_NUMBERS"] = "1"
        result = subprocess.run(["xvfb-run", "-a", "-s", "-screen 0 1920x1080x24", "dbus-run-session", "--",
                                 str(ROOT / "target/debug/nexuterm")], cwd=local, env=env, text=True,
                                capture_output=True, timeout=25)
        (artifacts / f"{tag}.log").write_text(result.stdout + result.stderr)
        assert result.returncode == 0 and "panicked at" not in result.stderr, result.stderr
        if external:
            assert (base / "opened.json").exists(), result.stderr
            assert json.loads((base / "opened.json").read_text()) == [str(nested / "sample con á.txt")]
            assert "NEXUTERM_SMOKE_LOCAL_EDITOR_OK" not in result.stderr
        markers = ["NEXUTERM_SMOKE_LOCAL_NAVIGATION_OK", "NEXUTERM_SMOKE_EXPLORER_RESIZE_OK", "NEXUTERM_SMOKE_PALETTE_SWITCH_OK"]
        if not external:
            markers.append("NEXUTERM_SMOKE_LOCAL_EDITOR_OK")
        for marker in markers:
            assert marker in result.stderr, result.stderr
        assert "Gtk-CRITICAL" not in result.stderr and "Theme parser error" not in result.stderr, result.stderr
        assert (artifacts / f"{tag}.png").exists()
        print(f"PASS {tag}: folder activation, file opening and editor preferences")


if __name__ == "__main__":
    check("dark")
    check("light", False)
    check("dark", external=True)
