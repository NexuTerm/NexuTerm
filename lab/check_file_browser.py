#!/usr/bin/env python3
"""Graphical FTP/SFTP navigation and transfers against disposable local fixtures."""
import argparse
import json
import logging
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import uuid

ROOT = Path(__file__).resolve().parent.parent
BODY = 'Texto de prueba UTF-8: áéíóú\n'.encode()
UPLOAD = b'NexuTerm graphical upload fixture\n'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('protocol', choices=['ftp', 'sftp'])
    parser.add_argument('--keyring', action='store_true')
    parser.add_argument('--password', action='store_true')
    parser.add_argument('--windows-ce', action='store_true')
    args = parser.parse_args()
    artifacts = ROOT / 'artifacts'
    artifacts.mkdir(exist_ok=True)
    upload_name = 'nexuterm-ui-' + uuid.uuid4().hex + '.txt'
    server = thread = None
    with tempfile.TemporaryDirectory(prefix='nexuterm-file-ui-') as directory:
        directory = Path(directory)
        local = directory / 'local'
        local.mkdir()
        (local / upload_name).write_bytes(UPLOAD)
        remote = directory / 'remote'
        fixtures = remote / 'fixtures'
        fixtures.mkdir(parents=True)
        (fixtures / 'subdir').mkdir()
        (fixtures / 'unicode.txt').write_bytes(BODY)
        config = directory / 'config' / 'nexuterm'
        config.mkdir(parents=True)
        (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[], preferences=dict(theme='dark'))))
        env = {k: v for k, v in os.environ.items() if not k.startswith('NEXUTERM_SMOKE') and k != 'NEXUTERM_SCREENSHOT'}
        runtime = directory / 'runtime'
        runtime.mkdir(mode=0o700)
        env.update(HOME=str(directory), TMPDIR=str(directory), XDG_CONFIG_HOME=str(config.parent),
                   XDG_DATA_HOME=str(directory / 'data'), XDG_RUNTIME_DIR=str(runtime),
                   GSK_RENDERER='cairo', NEXUTERM_SMOKE_TEST='1',
                   NEXUTERM_SMOKE_FILE_BROWSER=args.protocol,
                   NEXUTERM_SMOKE_FILE_BROWSER_LOCAL=str(local), NEXUTERM_SMOKE_UPLOAD_NAME=upload_name,
                   NEXUTERM_FILE_BROWSER_SCREENSHOT=str(artifacts / f'{args.protocol}-file-browser.png'))
        if args.protocol == 'ftp':
            from pyftpdlib.authorizers import DummyAuthorizer
            from pyftpdlib.handlers import FTPHandler
            from pyftpdlib.servers import FTPServer
            logging.getLogger('pyftpdlib').setLevel(logging.CRITICAL)
            authorizer = DummyAuthorizer()
            authorizer.add_user('demo', 'test only"\\fixture', str(remote), perm='elradfmwMT')
            class Handler(FTPHandler):
                def ftp_SYST(self, line):
                    if args.windows_ce:
                        self.respond('215 Windows_CE version 7.0.')
                    else:
                        super().ftp_SYST(line)

                def ftp_LIST(self, path):
                    if not args.windows_ce:
                        return super().ftp_LIST(path)
                    rows = []
                    for name in sorted(self.fs.listdir(path)):
                        item = Path(path) / name
                        kind = '<DIR>' if item.is_dir() else str(item.stat().st_size)
                        rows.append(f'09-29-22  04:05       {kind:>10}          {name}\r\n')
                    self.push_dtp_data(''.join(rows).encode(), isproducer=False, cmd='LIST')

            Handler.authorizer = authorizer
            server = FTPServer(('127.0.0.1', 0), Handler)
            port = server.socket.getsockname()[1]
            thread = threading.Thread(target=server.serve_forever,
                                      kwargs=dict(timeout=0.1, handle_exit=False), daemon=True)
            thread.start()
            env['NEXUTERM_SMOKE_BASIC'] = f'ftp:{port}'
        else:
            if not (ROOT / '.lab/known_hosts').exists():
                raise SystemExit('Run lab/lab.py up and lab/lab.py trust first')
            env.update(NEXUTERM_SMOKE_SSH_LAB='1', NEXUTERM_SMOKE_SFTP='1')
            if args.password:
                env['NEXUTERM_SMOKE_SFTP_PASSWORD'] = '1'
        command = [str(ROOT / 'target/debug/nexuterm')]
        if args.keyring:
            env['NEXUTERM_SMOKE_FTP_KEYRING'] = '1'
            command = ['python3', str(ROOT / 'lab/keyring_run.py'), *command]
        try:
            result = subprocess.run(['xvfb-run', '-a', '-s', '-screen 0 1280x820x24',
                                     'dbus-run-session', '--', *command], cwd=ROOT, env=env,
                                    capture_output=True, text=True, timeout=30)
            (artifacts / f'{args.protocol}-file-browser.log').write_text(result.stdout + result.stderr)
            assert result.returncode == 0, f'App failed: see artifacts/{args.protocol}-file-browser.log'
            assert 'NEXUTERM_SMOKE_FILE_BROWSER_OK' in result.stderr, f'UI check failed: see artifacts/{args.protocol}-file-browser.log'
            assert (local / 'unicode.txt').read_bytes() == BODY
            if args.protocol == 'ftp':
                assert (fixtures / upload_name).read_bytes() == UPLOAD
                assert 'NEXUTERM_SMOKE_FTP_WORKER_STOPPED' in result.stderr
            else:
                received = subprocess.check_output(['podman', 'exec', 'nexuterm-lab-' + str(os.getuid()),
                                                     'cat', '/home/tester/fixtures/' + upload_name])
                assert received == UPLOAD
            assert not list(directory.glob('nexuterm-ftp-*')), 'Graphical FTP must not create password files'
            print(f'PASS {args.protocol}: graphical local/remote panes, navigation, download, upload, no overwrite, close')
        finally:
            if server:
                server.close_all()
                thread.join(timeout=2)
            elif args.protocol == 'sftp':
                subprocess.run(['podman', 'exec', 'nexuterm-lab-' + str(os.getuid()),
                                'rm', '-f', '--', '/home/tester/fixtures/' + upload_name], check=True)


if __name__ == '__main__':
    main()
