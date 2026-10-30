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
import signal
import time

ROOT = Path(__file__).resolve().parent.parent
BODY = 'Texto de prueba UTF-8: áéíóú\n'.encode()
UPLOAD = b'NexuTerm graphical upload fixture\n'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('protocol', choices=['ftp', 'sftp'])
    parser.add_argument('--keyring', action='store_true')
    parser.add_argument('--password', action='store_true')
    parser.add_argument('--windows-ce', action='store_true')
    parser.add_argument('--queue', action='store_true', help='batch trees and durable recovery after SIGKILL')
    parser.add_argument('--colon-user', action='store_true', help='check percent-encoded FTP usernames')
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
        (local / 'drag-upload.txt').write_bytes(b'drag upload')
        remote = directory / 'remote'
        fixtures = remote / 'fixtures'
        fixtures.mkdir(parents=True)
        (fixtures / 'subdir').mkdir()
        (fixtures / 'unicode.txt').write_bytes(BODY)
        (fixtures / 'drag-download.txt').write_bytes(b'drag download')
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
                   GDK_BACKEND='x11', NEXUTERM_FILE_BROWSER_SCREENSHOT=str(artifacts / f'{args.protocol}-file-browser.png'))
        if args.protocol == 'ftp':
            from pyftpdlib.authorizers import DummyAuthorizer
            from pyftpdlib.handlers import FTPHandler
            from pyftpdlib.servers import FTPServer
            logging.getLogger('pyftpdlib').setLevel(logging.CRITICAL)
            authorizer = DummyAuthorizer()
            authorizer.add_user('demo:user' if args.colon_user else 'demo', 'test only"\\fixture', str(remote), perm='elradfmwMT')
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
            if args.colon_user:
                env['NEXUTERM_SMOKE_FTP_USER'] = 'demo:user'
        else:
            if not (ROOT / '.lab/known_hosts').exists():
                raise SystemExit('Run lab/lab.py up and lab/lab.py trust first')
            subprocess.run(['podman', 'exec', 'nexuterm-lab-' + str(os.getuid()),
                            'sh', '-c', 'printf "drag download" > /home/tester/fixtures/drag-download.txt'], check=True)
            env.update(NEXUTERM_SMOKE_SSH_LAB='1', NEXUTERM_SMOKE_SFTP='1')
            if args.password:
                env['NEXUTERM_SMOKE_SFTP_PASSWORD'] = '1'
        command = [str(ROOT / 'target/debug/nexuterm')]
        if args.keyring:
            env['NEXUTERM_SMOKE_FTP_KEYRING'] = '1'
            command = ['python3', str(ROOT / 'lab/keyring_run.py'), *command]
        queue_remote = None
        try:
            if args.queue:
                queue_remote = queue_gui(args, env, directory, local, remote, artifacts)
                return
            result = subprocess.run(['xvfb-run', '-a', '-s', '-screen 0 1280x820x24',
                                     'dbus-run-session', '--', *command], cwd=ROOT, env=env,
                                    capture_output=True, text=True, timeout=30)
            (artifacts / f'{args.protocol}-file-browser.log').write_text(result.stdout + result.stderr)
            assert result.returncode == 0, f'App failed: see artifacts/{args.protocol}-file-browser.log'
            assert 'NEXUTERM_SMOKE_FILE_BROWSER_OK' in result.stderr, f'UI check failed: see artifacts/{args.protocol}-file-browser.log'
            assert 'clipboard=both drag_drop=both single_local=1' in result.stderr
            assert (local / 'unicode.txt').read_bytes() == BODY
            assert (local / 'drag-download.txt').read_bytes() == b'drag download'
            if args.protocol == 'ftp':
                assert (fixtures / upload_name).read_bytes() == UPLOAD
                assert (fixtures / 'drag-upload.txt').read_bytes() == b'drag upload'
                assert 'NEXUTERM_SMOKE_FTP_WORKER_STOPPED' in result.stderr
            else:
                received = subprocess.check_output(['podman', 'exec', 'nexuterm-lab-' + str(os.getuid()),
                                                     'cat', '/home/tester/fixtures/' + upload_name])
                assert received == UPLOAD
            assert not list(directory.glob('nexuterm-ftp-*')), 'Graphical FTP must not create password files'
            print(f'PASS {args.protocol}: single local explorer, navigation, clipboard and drag/drop both ways, no overwrite, close')
        except subprocess.TimeoutExpired as error:
            (artifacts / f'{args.protocol}-file-browser.log').write_bytes((error.stdout or b'') + (error.stderr or b''))
            raise
        finally:
            if server:
                server.close_all()
                thread.join(timeout=2)
            elif args.protocol == 'sftp':
                if queue_remote:
                    subprocess.run(['podman', 'exec', 'nexuterm-lab-' + str(os.getuid()), 'rm', '-rf', '--', queue_remote], check=True)
                subprocess.run(['podman', 'exec', 'nexuterm-lab-' + str(os.getuid()),
                                'rm', '-f', '--', '/home/tester/fixtures/' + upload_name,
                                '/home/tester/fixtures/drag-download.txt', '/home/tester/fixtures/drag-upload.txt'], check=True)


def queue_gui(args, env, directory, local, remote, artifacts):
    for name in ('batch-a.txt', 'batch-b.txt', 'drag-a.txt', 'drag-b.txt', 'resume.txt'):
        (local / name).write_bytes(('queue fixture ' + name).encode())
    (local / 'folder tree/nested').mkdir(parents=True)
    (local / 'folder tree/empty').mkdir()
    (local / 'folder tree/nested/file %20 á.txt').write_text('nested body áé\n')
    (local / 'downloads').mkdir()
    token = 'nx-queue-' + uuid.uuid4().hex
    if args.protocol == 'ftp':
        (remote / 'fixtures' / token).mkdir()
        remote_path = '/fixtures/' + token
    else:
        remote_path = '/home/tester/fixtures/' + token
        subprocess.run(['podman', 'exec', '--user', 'tester', 'nexuterm-lab-' + str(os.getuid()), 'mkdir', '-m', '700', '--', remote_path], check=True)
    env = dict(env)
    env.pop('NEXUTERM_SMOKE_FILE_BROWSER', None)
    env.update(NEXUTERM_SMOKE_TRANSFER_QUEUE='batch', NEXUTERM_QUEUE_LOCAL=str(local),
               NEXUTERM_QUEUE_REMOTE=remote_path, NEXUTERM_QUEUE_SCREENSHOT=str(artifacts / f'{args.protocol}-queue.png'), XDG_STATE_HOME=str(directory / 'state'),
               XDG_CACHE_HOME=str(directory / 'cache'))
    process = subprocess.Popen(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')],
                               cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        deadline = time.monotonic() + 55
        marker = local / 'queue-ready'
        while not marker.exists() and process.poll() is None and time.monotonic() < deadline:
            time.sleep(.05)
        if not marker.exists():
            if process.poll() is None:
                process.kill()
            stdout, stderr = process.communicate(timeout=10)
            (artifacts / f'{args.protocol}-queue-batch.log').write_text(stdout + stderr)
            raise AssertionError(stderr)
        os.kill(int(marker.read_text()), signal.SIGKILL)
        stdout, stderr = process.communicate(timeout=10)
        (artifacts / f'{args.protocol}-queue-batch.log').write_text(stdout + stderr)
        assert 'NEXUTERM_SMOKE_TRANSFER_QUEUE_BATCH_OK' in stderr, stderr
        files = list((directory / 'data/nexuterm/transfer-queues').glob('*.json'))
        assert len(files) == 1
        serialized=files[0].read_text()
        assert 'test only' not in serialized and 'test%20only' not in serialized
        snapshot = json.loads(serialized)
        assert any(j['status'] == 'Pending' for j in snapshot['jobs'])
        assert any(j['status'] == 'Failed' for j in snapshot['jobs'])
        assert files[0].stat().st_mode & 0o777 == 0o600
        assert files[0].parent.stat().st_mode & 0o777 == 0o700
        env['NEXUTERM_SMOKE_TRANSFER_QUEUE'] = 'recover'
        result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--', str(ROOT / 'target/debug/nexuterm')],
                                cwd=ROOT, env=env, capture_output=True, text=True, timeout=55)
        (artifacts / f'{args.protocol}-queue-recover.log').write_text(result.stdout + result.stderr)
        assert result.returncode == 0 and 'NEXUTERM_SMOKE_TRANSFER_QUEUE_RECOVER_OK' in result.stderr, result.stderr
        assert 'panicked at' not in result.stderr and 'CRITICAL' not in result.stderr, result.stderr
        if args.protocol == 'ftp':
            assert (remote / 'fixtures' / token / 'batch-a.txt').read_bytes() == b'queue fixture batch-a.txt'
            assert (remote / 'fixtures' / token / 'resume.txt').read_bytes() == (local / 'resume.txt').read_bytes()
        else:
            assert subprocess.check_output(['podman', 'exec', 'nexuterm-lab-' + str(os.getuid()), 'cat', remote_path + '/batch-a.txt']) == b'queue fixture batch-a.txt'
            assert subprocess.check_output(['podman', 'exec', 'nexuterm-lab-' + str(os.getuid()), 'cat', remote_path + '/resume.txt']) == (local / 'resume.txt').read_bytes()
        print(f'PASS {args.protocol} queue: multi-select clipboard/drag both ways, trees, no overwrite, private journal, SIGKILL and explicit resume')
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate(timeout=10)
        if args.protocol == 'sftp':
            subprocess.run(['podman', 'exec', 'nexuterm-lab-' + str(os.getuid()), 'rm', '-rf', '--', remote_path], check=True)
    return None


if __name__ == '__main__':
    main()
