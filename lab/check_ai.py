#!/usr/bin/env python3
"""GTK agent checks with fake CLI and loopback APIs; never calls a real model.

Uses xvfb-run if available, otherwise the current DISPLAY. Config/data live in /tmp.
"""
import http.server
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading

ROOT = Path(__file__).resolve().parent.parent
ARTIFACTS = ROOT / "artifacts"


def run(mode, profile, directory, profiles=None):
    config = directory / "config" / "nexuterm"
    config.mkdir(parents=True)
    path = config / "sessions.json"
    path.write_text(json.dumps(dict(version=2, profiles=[], ai_agents=profiles if profiles is not None else [profile], preferences=dict(theme="dark" if mode.startswith("codex") else "system"))))
    env = {k: v for k, v in os.environ.items()
           if not k.startswith("NEXUTERM_SMOKE") and k not in ("NEXUTERM_SCREENSHOT", "NEXUTERM_AI_SETTINGS_SCREENSHOT")}
    env.update(HOME=str(directory), XDG_CONFIG_HOME=str(config.parent),
               XDG_DATA_HOME=str(directory / "data"), XDG_CACHE_HOME=str(directory / "cache"),
               XDG_STATE_HOME=str(directory / "state"), NEXUTERM_SMOKE_TEST="1",
               NEXUTERM_SMOKE_AI=mode, GSK_RENDERER="cairo", PATH=str(directory / "bin") + os.pathsep + env["PATH"],
               NEXUTERM_SCREENSHOT=str(ARTIFACTS / f"ai-{profile['id']}.png"),
               NEXUTERM_AI_SETTINGS_SCREENSHOT=str(ARTIFACTS / "ai-settings-dialog.png"))
    if mode == "login":
        env.update(PATH=str(directory / "bin") + os.pathsep + env["PATH"],
                   BROWSER=str(directory / "bin" / "fixture-browser"),
                   CODEX_HOME=str(directory / "codex-home"))
    application = [str(ROOT / "target/debug/nexuterm")]
    if mode == "keyring":
        application = [shutil.which("python3"), str(ROOT / "lab/keyring_run.py"), *application]
    command = ["dbus-run-session", "--", *application]
    if shutil.which("xvfb-run"):
        command = ["xvfb-run", "-a", *command]
    elif not env.get("DISPLAY"):
        raise RuntimeError("Requires xvfb-run or an available DISPLAY")
    try:
        result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=20)
    except subprocess.TimeoutExpired as error:
        (ARTIFACTS / f"ai-{profile['id']}.log").write_bytes((error.stdout or b"") + (error.stderr or b""))
        raise
    (ARTIFACTS / f"ai-{profile['id']}.log").write_text(result.stdout + result.stderr)
    assert result.returncode == 0, result.stderr
    assert "CRITICAL" not in result.stderr, result.stderr
    return json.loads(path.read_text()), result.stderr


def check_api(kind):
    events = []
    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            messages = body["input" if kind == "Responses" else "messages"]
            events.append((self.path, body))
            index = len(events)
            assert messages[-1]["content"] == ("primer-mensaje" if index == 1 else "segundo-mensaje")
            if index == 2:
                assert messages[-2] == dict(role="assistant", content="fixture-1")
                assert messages[-3] == dict(role="user", content="primer-mensaje")
            if kind == "Responses":
                assert body["store"] is False
                reply = dict(status="completed", output=[dict(type="message", content=[dict(type="output_text", text=f"fixture-{index}")])])
            else:
                reply = dict(choices=[dict(message=dict(content=f"fixture-{index}"))])
            data = json.dumps(reply).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        with tempfile.TemporaryDirectory(prefix="nexuterm-ai-api-") as directory:
            profile = dict(id=kind.lower(), name=f"API {kind} de prueba", kind=kind,
                           endpoint=f"http://127.0.0.1:{server.server_port}/v1", model="fixture-model")
            _, logs = run("api", profile, Path(directory))
            assert "NEXUTERM_SMOKE_AI_API_OK" in logs, logs
            assert len(events) == 2, events
            print(f"PASS native {kind}: two chat turns, preserved context")
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)


def check_terminals():
    with tempfile.TemporaryDirectory(prefix="nexuterm-ai-terminals-") as directory:
        profile = dict(id="terminals", name="Terminales de prueba", kind="Responses",
                       endpoint="http://127.0.0.1:1/v1", model="fixture")
        _, logs = run("terminals", profile, Path(directory))
        assert "NEXUTERM_SMOKE_TERMINALS_OK" in logs, logs
        print("PASS terminal bridge: two simulated sessions, explicit execution/read approval")


def check_shutdown():
    closed = threading.Event()
    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_POST(self):
            self.rfile.read(int(self.headers["Content-Length"]))
            self.connection.settimeout(6)
            try:
                if self.rfile.read(1) == b"":
                    closed.set()
            except ConnectionResetError:
                closed.set()
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        with tempfile.TemporaryDirectory(prefix="nexuterm-ai-shutdown-") as directory:
            profile = dict(id="shutdown", name="Solicitud pendiente", kind="Responses",
                           endpoint=f"http://127.0.0.1:{server.server_port}/v1", model="fixture-model")
            run("shutdown", profile, Path(directory))
            assert closed.wait(2), "application exit left the HTTP transport alive"
            print("PASS shutdown: pending HTTP transport closes on application exit")
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)


def check_usage():
    with tempfile.TemporaryDirectory(prefix="nexuterm-usage-") as directory:
        directory=Path(directory)
        codex=directory/"codex"
        codex.write_text("#!"+shutil.which("python3")+"\n"+(ROOT/"lab/codex_fixture.py").read_text());codex.chmod(0o700)
        profile=dict(id="usage",name="Codex",kind="Codex",program=str(codex),directory=str(directory))
        _,logs=run("usage",profile,directory)
        assert "NEXUTERM_SMOKE_USAGE_OK" in logs,logs
        print("PASS usage: account name, 5-hour remaining 75%, weekly remaining 40% from provider")


def check_new_chat():
    with tempfile.TemporaryDirectory(prefix="nexuterm-new-chat-") as directory:
        directory=Path(directory)
        codex=directory/"codex"
        codex.write_text("#!"+shutil.which("python3")+"\n"+(ROOT/"lab/codex_fixture.py").read_text());codex.chmod(0o700)
        acp=directory/"acp"
        acp.write_text("#!"+shutil.which("python3")+"\n"+(ROOT/"lab/acp_fixture.py").read_text());acp.chmod(0o700)
        profiles=[dict(id="original-codex",name="Codex",kind="Codex",program=str(codex),directory=str(directory)),
                  dict(id="second-copilot",name="GitHub Copilot",kind="Acp",program=str(acp),directory=str(directory),model="fixture/two",models=["fixture/one","fixture/two"])]
        _,logs=run("new-chat",profiles[0],directory,profiles=profiles)
        assert "NEXUTERM_SMOKE_NEW_CHAT_OK" in logs,logs
        requests=[json.loads(line) for line in (directory/"acp-protocol.jsonl").read_text().splitlines()]
        assert any(r.get("method")=="session/prompt" for r in requests)
        assert any(r.get("method")=="session/set_config_option" and r["params"]["value"]=="fixture/two" for r in requests)
        print("PASS new chat: choose agent/model, second response, original draft/client preserved, close independently")


def check_local(switch=False):
    with tempfile.TemporaryDirectory(prefix="nexuterm-acp-gui-") as directory:
        directory=Path(directory)
        cli=directory/"acp"
        cli.write_text("#!"+shutil.which("python3")+"\n"+(ROOT/"lab/acp_fixture.py").read_text());cli.chmod(0o700)
        profile=dict(id="acp",name="OpenCode",kind="Acp",program=str(cli),directory=str(directory),model="fixture/two",models=["fixture/one","fixture/two"])
        profiles=None
        if switch:
            codex=directory/"codex";codex.write_text("#!"+shutil.which("python3")+"\n"+(ROOT/"lab/codex_fixture.py").read_text());codex.chmod(0o700)
            profiles=[dict(id="first-codex",name="Codex",kind="Codex",program=str(codex),directory=str(directory)),profile]
        config,logs=run("switch" if switch else "acp",profile,directory,profiles=profiles)
        assert "NEXUTERM_SMOKE_ACP_OK" in logs,logs
        requests=[json.loads(line) for line in (directory/"acp-protocol.jsonl").read_text().splitlines()]
        assert any(r.get("method")=="session/set_config_option" and r["params"]["value"]=="fixture/two" for r in requests)
        assert len(config["ai_agents"])==(2 if switch else 1)
        print("PASS shared graphical chat: ACP models"+("; Codex → OpenCode selector" if switch else ""))


def check_settings():
    with tempfile.TemporaryDirectory(prefix="nexuterm-ai-settings-") as directory:
        profile = dict(id="settings", name="OpenCode", kind="OpenCode", program="opencode")
        config, _ = run("settings", profile, Path(directory))
        agent = config["ai_agents"][0]
        assert agent["name"] == "OpenCode comprobado"
        extensions = agent["extensions"]
        assert extensions["mcp"]["servidor-1"]["enabled"] is False
        assert len(extensions["skills"]["paths"]) == 1
        assert {a["mode"] for a in extensions["agent"].values()} == {"primary", "subagent"}
        assert (ARTIFACTS / "ai-settings-dialog.png").exists()
        print("PASS settings: saved MCP, skill path, primary agent and subagent")


def check_settings_capabilities():
    with tempfile.TemporaryDirectory(prefix="nexuterm-settings-capabilities-") as directory:
        profile = dict(id="api-capabilities", name="Modelo local", kind="ChatCompletions", endpoint="http://127.0.0.1:11434/v1", model="fixture")
        config, logs = run("settings-api", profile, Path(directory))
        assert "NEXUTERM_SMOKE_CAPABILITIES_OK" in logs, logs
        assert config["ai_agents"][0]["kind"] == "ChatCompletions"
        assert config["ai_agents"][0]["model"] == "fixture"
        print("PASS settings capabilities: compatible engine only, API fields, saved connector unchanged")


def check_groups():
    with tempfile.TemporaryDirectory(prefix="nexuterm-ai-groups-") as directory:
        profiles = [dict(id="web-group", name="Web de prueba", kind="Codex", program="/usr/bin/false"),
                    dict(id="local-group", name="Local de prueba", kind="ChatCompletions", endpoint="http://127.0.0.1:11434/v1", model="fixture")]
        config, logs = run("groups", profiles[0], Path(directory), profiles=profiles)
        assert "NEXUTERM_SMOKE_MODEL_GROUPS_OK" in logs, logs
        assert [a["id"] for a in config["ai_agents"]] == ["web-group", "local-group"]
        print("PASS grouped settings: Locales/Web, add-model button, no profile mutations")


def check_templates():
    with tempfile.TemporaryDirectory(prefix="nexuterm-ai-template-") as directory:
        directory = Path(directory)
        binary = directory / "bin"
        binary.mkdir()
        cli = binary / "opencode"
        cli.write_text("#!" + shutil.which("python3") + "\n" + (ROOT / "lab/acp_fixture.py").read_text())
        cli.chmod(0o700)
        profile = dict(id="template-original", name="Codex conservado", kind="Codex", program="codex")
        config, logs = run("templates", profile, directory)
        assert len(config["ai_agents"]) == 2, config
        assert config["ai_agents"][0]["id"] == profile["id"] and config["ai_agents"][0]["name"] == profile["name"], config
        assert config["ai_agents"][1]["kind"] == "OpenCode", config
        protocol = list((directory / "data").rglob("acp-protocol.jsonl"))
        # The configured default cwd is the isolated HOME, not the profile storage.
        protocol += list(directory.glob("acp-protocol.jsonl"))
        assert protocol, logs
        requests = [json.loads(line) for line in protocol[0].read_text().splitlines()]
        assert any(r.get("method") == "session/new" for r in requests), requests
        print("PASS template: preserves Codex, adds OpenCode and opens its graphical ACP chat")


def check_codex_login(existing, cached=True):
    with tempfile.TemporaryDirectory(prefix="nexuterm-codex-native-") as directory:
        directory = Path(directory)
        binary = directory / "bin"
        binary.mkdir()
        cli = binary / "codex"
        cli.write_text("#!" + shutil.which("python3") + "\n" + (ROOT / "lab/codex_fixture.py").read_text())
        cli.chmod(0o700)
        profile = dict(id="codex-existing" if existing else "codex-new", name="Codex gráfico",
                       kind="Local", program=str(cli),
                       args=["ignored-tui-argument"] if existing else [], directory=str(directory))
        # New automatically created profiles default to HOME, isolated by run().
        config, logs = run("login", profile, directory)
        requests = [json.loads(line) for line in (directory / "protocol.jsonl").read_text().splitlines()]
        assert "NEXUTERM_SMOKE_CODEX_CACHED_OK" in logs, logs
        assert not any(r.get("method") == "account/login/start" for r in requests)
        assert len(config["ai_agents"]) == 1
        assert not (directory / "codex-home/auth.json").exists()
        print("PASS native Codex login: cached account, no browser or console, existing=" + str(existing))


def check_codex_chat(approval=False):
    with tempfile.TemporaryDirectory(prefix="nexuterm-codex-chat-") as directory:
        directory = Path(directory)
        cli = directory / "codex"
        cli.write_text("#!" + shutil.which("python3") + "\n" + (ROOT / "lab/codex_fixture.py").read_text())
        cli.chmod(0o700)
        profile = dict(id="codex-approval" if approval else "codex", name="Codex", kind="Local", program=str(cli), directory=str(directory))
        (directory / "context.txt").write_text("Contexto ficticio")
        _, logs = run("codex-approval" if approval else "codex", profile, directory)
        if approval:
            assert "NEXUTERM_SMOKE_CODEX_APPROVAL_REJECTED" in logs, logs
        else:
            assert "NEXUTERM_SMOKE_CODEX_NATIVE_OK" in logs, logs
        requests = [json.loads(line) for line in (directory / "protocol.jsonl").read_text().splitlines()]
        turn = next(r for r in requests if r.get("method") == "turn/start")
        assert turn["params"]["model"] == "fixture-model"
        assert turn["params"]["effort"] == "low"
        if approval:
            assert any(r.get("id") == "approval-1" and r.get("result", {}).get("decision") == "decline" for r in requests)
        else:
            assert "context.txt" in turn["params"]["input"][0]["text"]
        print("PASS graphical Codex: legacy profile, native composer, streamed response")


def check_credentials(remember):
    mode = "keyring" if remember else "credentials"
    with tempfile.TemporaryDirectory(prefix="nexuterm-ai-credentials-") as directory:
        profile = dict(id=mode, name="Credenciales ficticias", kind="Responses",
                       endpoint="http://127.0.0.1:11434/v1", model="fixture-model")
        config, logs = run(mode, profile, Path(directory))
        assert "NEXUTERM_SMOKE_AI_CREDENTIALS_OK" in logs, logs
        assert config["ai_agents"][0]["secret_ref"] is None
        print(f"PASS {mode}: key storage, endpoint isolation, explicit deletion, no plaintext in JSON")


def check_opencode():
    program = shutil.which("opencode")
    if not program:
        print("SKIP OpenCode CLI: not installed")
        return
    with tempfile.TemporaryDirectory(prefix="nexuterm-opencode-") as directory:
        directory = Path(directory)
        skill = directory / "skills" / "fixture-skill"
        skill.mkdir(parents=True)
        (skill / "SKILL.md").write_text("---\nname: fixture-skill\ndescription: Fictitious integration skill\n---\nUse only fictitious data.\n")
        marker = directory / "mcp-initialized"
        server = directory / "mcp.py"
        server.write_text(
            "import sys,json,pathlib\n"
            "for line in sys.stdin:\n"
            " request=json.loads(line)\n"
            " if 'id' not in request: continue\n"
            " if request.get('method')=='initialize':\n"
            "  pathlib.Path(sys.argv[1]).write_text('initialized')\n"
            "  result={'protocolVersion':'2024-11-05','capabilities':{'tools':{}},'serverInfo':{'name':'fixture','version':'1'}}\n"
            " else: result={'tools':[]}\n"
            " print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result}),flush=True)\n"
        )
        config = dict(
            mcp={"fixture-mcp": dict(type="local", command=[shutil.which("python3"), str(server), str(marker)], enabled=True)},
            skills=dict(paths=[str(directory / "skills")]),
            agent={"fixture-main": dict(description="Primary fixture", mode="primary", prompt="Fictitious data only"),
                   "fixture-review": dict(description="Review fixture", mode="subagent", prompt="Review fictitious data only")},
        )
        clean_env = {k: v for k, v in os.environ.items() if not k.startswith("OPENCODE_")}
        env = dict(clean_env, HOME=str(directory), XDG_CONFIG_HOME=str(directory / "config"),
                   XDG_DATA_HOME=str(directory / "data"), XDG_CACHE_HOME=str(directory / "cache"),
                   XDG_STATE_HOME=str(directory / "state"), OPENCODE_DISABLE_AUTOUPDATE="1",
                   OPENCODE_CONFIG_CONTENT=json.dumps(config))
        def execute(*args):
            result = subprocess.run([program, *args, "--pure"], cwd=directory, env=env,
                                    capture_output=True, text=True, timeout=20)
            assert result.returncode == 0, result.stderr
            return result.stdout
        skills = execute("debug", "skill")
        assert "fixture-skill" in skills, skills
        agent = execute("debug", "agent", "fixture-review")
        assert '"mode": "subagent"' in agent, agent
        primary = execute("debug", "agent", "fixture-main")
        assert '"mode": "primary"' in primary, primary
        execute("mcp", "list")
        assert marker.read_text() == "initialized"
        print("PASS installed OpenCode: native MCP handshake, skill discovery, primary agent and subagent")


def check_logout():
    with tempfile.TemporaryDirectory(prefix="nexuterm-logout-gui-") as directory:
        directory = Path(directory)
        cli = directory / "codex"
        cli.write_text("#!" + shutil.which("python3") + "\n" + (ROOT / "lab/codex_fixture.py").read_text())
        cli.chmod(0o700)
        profile = dict(id="codex-logout", name="Codex", kind="Codex", program=str(cli), directory=str(directory))
        _, logs = run("logout", profile, directory)
        assert "NEXUTERM_SMOKE_LOGOUT_OK" in logs, logs
        requests = [json.loads(line) for line in (directory / "protocol.jsonl").read_text().splitlines()]
        assert any(r.get("method") == "account/logout" for r in requests)
        print("PASS graphical logout: link and restored login")


def check_copilot_account():
    import hashlib
    with tempfile.TemporaryDirectory(prefix="nexuterm-copilot-account-") as directory:
        directory = Path(directory)
        cli = directory / "copilot"
        cli.write_text("#!" + shutil.which("python3") + "\n" + (ROOT / "lab/acp_fixture.py").read_text())
        cli.chmod(0o700)
        profile = dict(id="copilot-account", name="GitHub Copilot", kind="Acp", program=str(cli), args=["--acp"], directory=str(directory))
        account = directory / "data/nexuterm/agents" / hashlib.sha256(profile["id"].encode()).hexdigest() / ".copilot/config.json"
        account.parent.mkdir(parents=True)
        account.write_text('// Provider-managed state\n' + json.dumps(dict(lastLoggedInUser=dict(login="fixture-user"))))
        _, logs = run("provider-auth", profile, directory)
        assert "NEXUTERM_SMOKE_PROVIDER_AUTH_HIDDEN" in logs, logs
        print("PASS graphical Copilot account: remembered account hides login")


if __name__ == "__main__":
    ARTIFACTS.mkdir(exist_ok=True)
    check_logout()
    check_copilot_account()
    check_usage()
    check_new_chat()
    check_local()
    check_local(switch=True)
    check_codex_login(True)
    check_codex_login(False)
    check_codex_chat()
    check_codex_chat(approval=True)
    check_api("Responses")
    check_api("ChatCompletions")
    check_shutdown()
    check_terminals()
    check_settings()
    check_templates()
    check_groups()
    check_settings_capabilities()
    check_credentials(False)
    check_credentials(True)
    check_opencode()
