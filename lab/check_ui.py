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
    parser.add_argument("scenario", choices=["tabs", "colors", "sessions", "ssh", "splits", "close-splits", "menus", "rdp", "rdp-certificate", "sftp"])
    parser.add_argument("--compact", action="store_true")
    parser.add_argument("--scale", choices=[1, 2], default=1, type=int)
    parser.add_argument("--theme", choices=["dark", "light"], default="dark")
    parser.add_argument("--workspace", action="store_true", help="restore a saved SSH/RDP workspace before connecting")
    args = parser.parse_args()
    if args.workspace and args.scenario not in ("ssh", "rdp", "rdp-certificate"):
        parser.error("--workspace requires ssh or rdp")
    artifacts = ROOT / "artifacts"
    artifacts.mkdir(exist_ok=True)
    width, height = (1024, 768) if args.compact else (1280, 820)
    tag = "{}-{}-scale{}".format(args.scenario, width, args.scale)
    if args.scenario in ("colors", "tabs"):
        tag += "-" + args.theme
    screenshot = artifacts / (tag + ".png")
    env = {k: v for k, v in os.environ.items() if not k.startswith("NEXUTERM_SMOKE") and k != "NEXUTERM_SCREENSHOT"}
    with tempfile.TemporaryDirectory(prefix="nexuterm-ui-") as directory:
        config = pathlib.Path(directory) / "nexuterm"
        config.mkdir()
        protocols = [("Servidor de desarrollo", "Ssh", 22), ("Escritorio remoto", "Rdp", 3389), ("Pantalla de pruebas", "Vnc", 5900)]
        profiles = [dict(id=str(i), name=name, protocol=protocol, host="demo.example.invalid", port=port, username="demo", group="Laboratorio", favorite=i == 0) for i, (name, protocol, port) in enumerate(protocols)]
        (config / "sessions.json").write_text(json.dumps(dict(version=2, profiles=profiles, preferences=dict(theme=args.theme))))
        env.update(XDG_CONFIG_HOME=directory, XDG_DATA_HOME=str(pathlib.Path(directory) / "data"), NEXUTERM_SMOKE_TEST="1", NEXUTERM_SCREENSHOT=str(screenshot), GSK_RENDERER="cairo", GDK_BACKEND="x11", GDK_SCALE=str(args.scale))
        if args.compact:
            env["NEXUTERM_SMOKE_COMPACT"] = "1"
        marker = "NEXUTERM_SMOKE_CHILD_EXIT=0"
        if args.scenario == "tabs":
            env["NEXUTERM_SMOKE_TABS"] = "1"
            marker = "NEXUTERM_SMOKE_TABS_OK"
        elif args.scenario == "colors":
            env["NEXUTERM_SMOKE_COLORS"] = "1"
            marker = "NEXUTERM_SMOKE_COLORS_OK"
        elif args.scenario == "sessions":
            env.update(NEXUTERM_SMOKE_SESSIONS="1", NEXUTERM_SMOKE_SESSION_CRUD="1")
            marker = "NEXUTERM_SMOKE_SESSIONS_OK"
        elif args.scenario == "menus":
            env["NEXUTERM_SMOKE_MENUS"] = "1"
            marker = "NEXUTERM_SMOKE_MENUS_OK"
        elif args.scenario in ("rdp", "rdp-certificate"):
            env["NEXUTERM_SMOKE_RDP"] = "1"
            env["NEXUTERM_RDP_SCREENSHOT"] = str(artifacts / "rdp-desktop.png")
            marker = "NEXUTERM_SMOKE_RDP_OK"
            if args.scenario == "rdp-certificate":
                env["NEXUTERM_SMOKE_RDP_CERTIFICATE"] = "1"
                marker = "NEXUTERM_SMOKE_RDP_CERTIFICATE_CANCELLED"
        elif args.scenario in ("ssh", "sftp"):
            if not (ROOT / ".lab/known_hosts").exists():
                raise SystemExit("Run lab/lab.py up and lab/lab.py trust first")
            env["NEXUTERM_SMOKE_SSH_LAB"] = "1"
            marker = "NEXUTERM_SMOKE_REMOTE_READY"
            if args.scenario == "sftp":
                env["NEXUTERM_SMOKE_SFTP"] = "1"
                marker = "NEXUTERM_SMOKE_SFTP_AUTH_HIDDEN"
        else:
            env["NEXUTERM_SMOKE_ACTIONS"] = "1"
            marker = "NEXUTERM_SMOKE_ACTIONS_OK"
            if args.scenario == "close-splits":
                env["NEXUTERM_SMOKE_CLOSE_SPLITS"] = "1"
                marker = "NEXUTERM_SMOKE_CLOSE_SPLITS_OK"
        if args.workspace:
            tag += "-workspace"
            screenshot = artifacts / (tag + ".png")
            env["NEXUTERM_SCREENSHOT"] = str(screenshot)
            if args.scenario == "ssh":
                env["NEXUTERM_SMOKE_WORKSPACE_SSH"] = "1"
                marker = "NEXUTERM_SMOKE_WORKSPACE_SSH_OK"
            else:
                env["NEXUTERM_SMOKE_RDP_WORKSPACE"] = "1"
        result = subprocess.run(["xvfb-run", "-a", "-s", "-screen 0 {}x{}x24".format(width * args.scale, height * args.scale), "dbus-run-session", "--", str(ROOT / "target/debug/nexuterm")], env=env, cwd=ROOT, capture_output=True, text=True, timeout=30)
        (artifacts / (tag + ".log")).write_text(result.stdout + result.stderr)
        workspace_marker_ok = not args.workspace or args.scenario == "ssh" or "NEXUTERM_SMOKE_RDP_WORKSPACE_OK" in result.stderr
        if result.returncode or marker not in result.stderr or not screenshot.exists() or not workspace_marker_ok:
            raise SystemExit("UI check failed; see artifacts/{}.log".format(tag))
        print("PASS {} — {}".format(tag, screenshot))


if __name__ == "__main__":
    main()
