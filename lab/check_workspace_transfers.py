#!/usr/bin/env python3
"""Verify offline FTP/SFTP workspace restoration without contacting remote hosts."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def main():
    with tempfile.TemporaryDirectory(prefix="nexuterm-workspace-transfers-") as temporary:
        directory = Path(temporary)
        config = directory / "config/nexuterm"
        config.mkdir(parents=True)
        (config / "sessions.json").write_text(json.dumps(dict(version=2, profiles=[])))
        env = {k: v for k, v in os.environ.items()
               if not k.startswith("NEXUTERM_SMOKE") and k != "NEXUTERM_SCREENSHOT"}
        env.update(HOME=temporary, XDG_CONFIG_HOME=str(config.parent),
                   XDG_DATA_HOME=str(directory / "data"), XDG_STATE_HOME=str(directory / "state"),
                   XDG_CACHE_HOME=str(directory / "cache"), NEXUTERM_SMOKE_TEST="1",
                   NEXUTERM_SMOKE_TRANSFER_RESTORE="1", GSK_RENDERER="cairo",
                   GDK_BACKEND="x11", SHELL="/bin/sh")
        result = subprocess.run(["xvfb-run", "-a", "dbus-run-session", "--",
                                 str(ROOT / "target/debug/nexuterm")],
                                cwd=directory, env=env, capture_output=True, text=True, timeout=20)
        artifacts = ROOT / "artifacts"
        artifacts.mkdir(exist_ok=True)
        (artifacts / "workspace-transfers.log").write_text(result.stdout + result.stderr)
        assert result.returncode == 0, result.stderr
        assert "CRITICAL" not in result.stderr, result.stderr
        assert "NEXUTERM_SMOKE_TRANSFER_RESTORE_OK" in result.stderr, result.stderr
        print("PASS FTP/SFTP restoration: order, paths, offline state and missing/changed profiles")


if __name__ == "__main__":
    main()
