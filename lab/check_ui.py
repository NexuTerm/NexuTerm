#!/usr/bin/env python3
"""Isolated GTK checks with fictitious profiles; SSH mode only uses our loopback lab."""
import argparse
import json
import os
import pathlib
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("scenario", choices=["sessions", "ssh", "splits", "menus"])
    parser.add_argument("--compact", action="store_true")
    parser.add_argument("--scale", choices=[1, 2], default=1, type=int)
    args = parser.parse_args()
    artifacts = ROOT / "artifacts"
    artifacts.mkdir(exist_ok=True)
    width, height = (1024, 768) if args.compact else (1280, 820)
    tag = "{}-{}-scale{}".format(args.scenario, width, args.scale)
    screenshot = artifacts / (tag + ".png")
    env = {k: v for k, v in os.environ.items() if not k.startswith("NEXUTERM_SMOKE") and k != "NEXUTERM_SCREENSHOT"}
    with tempfile.TemporaryDirectory(prefix="nexuterm-ui-") as directory:
        config = pathlib.Path(directory) / "nexuterm"
        config.mkdir()
        protocols = [("Servidor de desarrollo", "Ssh", 22), ("Escritorio remoto", "Rdp", 3389), ("Pantalla de pruebas", "Vnc", 5900)]
        profiles = [dict(id=str(i), name=name, protocol=protocol, host="demo.example.invalid", port=port, username="demo", group="Laboratorio", favorite=i == 0) for i, (name, protocol, port) in enumerate(protocols)]
        (config / "sessions.json").write_text(json.dumps(dict(version=2, profiles=profiles, preferences=dict(theme="dark"))))
        env.update(XDG_CONFIG_HOME=directory, NEXUTERM_SMOKE_TEST="1", NEXUTERM_SCREENSHOT=str(screenshot), GSK_RENDERER="cairo", GDK_SCALE=str(args.scale))
        if args.compact:
            env["NEXUTERM_SMOKE_COMPACT"] = "1"
        marker = "NEXUTERM_SMOKE_CHILD_EXIT=0"
        if args.scenario == "sessions":
            env.update(NEXUTERM_SMOKE_SESSIONS="1", NEXUTERM_SMOKE_SESSION_CRUD="1")
            marker = "NEXUTERM_SMOKE_SESSIONS_OK"
        elif args.scenario == "menus":
            env["NEXUTERM_SMOKE_MENUS"] = "1"
            marker = "NEXUTERM_SMOKE_MENUS_OK"
        elif args.scenario == "ssh":
            if not (ROOT / ".lab/known_hosts").exists():
                raise SystemExit("Run lab/lab.py up and lab/lab.py trust first")
            env["NEXUTERM_SMOKE_SSH_LAB"] = "1"
            marker = "NEXUTERM_SMOKE_REMOTE_READY"
        else:
            env["NEXUTERM_SMOKE_ACTIONS"] = "1"
            marker = "NEXUTERM_SMOKE_ACTIONS_OK"
        result = subprocess.run(["xvfb-run", "-a", "-s", "-screen 0 {}x{}x24".format(width * args.scale, height * args.scale), "dbus-run-session", "--", str(ROOT / "target/debug/nexuterm")], env=env, cwd=ROOT, capture_output=True, text=True, timeout=30)
        (artifacts / (tag + ".log")).write_text(result.stdout + result.stderr)
        if result.returncode or marker not in result.stderr or not screenshot.exists():
            raise SystemExit("UI check failed; see artifacts/{}.log".format(tag))
        print("PASS {} — {}".format(tag, screenshot))


if __name__ == "__main__":
    main()
