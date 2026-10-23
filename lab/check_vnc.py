#!/usr/bin/env python3
"""Test embedded VNC using a disposable RFB 3.8 server bound to loopback only."""
import json
import argparse
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import threading

ROOT = Path(__file__).resolve().parent.parent


def receive(connection, length):
    data = bytearray()
    while len(data) < length:
        chunk = connection.recv(length - len(data))
        if not chunk:
            raise EOFError()
        data.extend(chunk)
    return bytes(data)


def serve(listener, events, failures, password_auth):
    try:
        connection, address = listener.accept()
        assert address[0] == '127.0.0.1'
        with connection:
            connection.settimeout(8)
            connection.sendall(b'RFB 003.008\n')
            assert receive(connection, 12) == b'RFB 003.008\n'
            security = b'\x02' if password_auth else b'\x01'
            connection.sendall(b'\x01' + security)
            assert receive(connection, 1) == security
            if password_auth:
                connection.sendall(b'\x5a' * 16)
                response = receive(connection, 16)
                assert response != b'\x00' * 16
                events.append(('password-response',))
            connection.sendall(struct.pack('!I', 0))
            receive(connection, 1)  # ClientInit: shared flag.
            pixel_format = struct.pack('!BBBBHHHBBB3x', 32, 24, 0, 1, 255, 255, 255, 0, 8, 16)
            name = b'NexuTerm embedded VNC lab'
            connection.sendall(struct.pack('!HH', 64, 48) + pixel_format
                               + struct.pack('!I', len(name)) + name)
            while True:
                try:
                    kind = receive(connection, 1)[0]
                except EOFError:
                    events.append(('closed',))
                    return
                if kind == 0:
                    requested = receive(connection, 19)
                    assert requested[3:7] == bytes([32, 24, 0, 1])
                elif kind == 2:
                    header = receive(connection, 3)
                    receive(connection, struct.unpack('!H', header[1:])[0] * 4)
                elif kind == 3:
                    receive(connection, 9)
                    if ('frame',) not in events:
                        rgba = bytes([32, 96, 160, 0]) * (64 * 48)
                        connection.sendall(struct.pack('!BBHHHHHi', 0, 0, 1, 0, 0, 64, 48, 0) + rgba)
                        events.append(('frame',))
                elif kind == 4:
                    message = receive(connection, 7)
                    events.append(('key', message[0], struct.unpack('!I', message[3:])[0]))
                elif kind == 5:
                    message = receive(connection, 5)
                    x, y = struct.unpack('!HH', message[1:])
                    events.append(('pointer', message[0], x, y))
                elif kind == 6:
                    header = receive(connection, 7)
                    length = struct.unpack('!I', header[3:])[0]
                    assert length <= 1048576
                    receive(connection, length)
                else:
                    raise AssertionError(f'Unexpected client message: {kind}')
    except Exception as error:
        failures.append(error)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--password', action='store_true', help='exercise the VNC password challenge')
    args = parser.parse_args()
    artifacts = ROOT / 'artifacts'
    artifacts.mkdir(exist_ok=True)
    events, failures = [], []
    with socket.socket() as listener, tempfile.TemporaryDirectory(prefix='nexuterm-vnc-') as directory:
        listener.bind(('127.0.0.1', 0))
        listener.listen(1)
        listener.settimeout(12)
        worker = threading.Thread(target=serve, args=(listener, events, failures, args.password), daemon=True)
        worker.start()
        config = Path(directory) / 'nexuterm'
        config.mkdir()
        (config / 'sessions.json').write_text(json.dumps(dict(version=2, profiles=[], preferences=dict(theme='dark'))))
        env = {k: v for k, v in os.environ.items() if not k.startswith('NEXUTERM_SMOKE') and k != 'NEXUTERM_SCREENSHOT'}
        env.update(XDG_CONFIG_HOME=directory, NEXUTERM_SMOKE_TEST='1',
                   NEXUTERM_SMOKE_VNC=str(listener.getsockname()[1]),
                   NEXUTERM_VNC_SCREENSHOT=str(artifacts / 'vnc-integrated.png'), GSK_RENDERER='cairo')
        if args.password:
            env['NEXUTERM_SMOKE_VNC_PASSWORD'] = '1'
        result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--',
                                 str(ROOT / 'target/debug/nexuterm')], cwd=ROOT,
                                env=env, capture_output=True, text=True, timeout=20)
        (artifacts / 'vnc-integrated.log').write_text(result.stdout + result.stderr)
        worker.join(3)
        assert not worker.is_alive(), 'VNC worker did not stop'
        assert not failures, failures
        assert result.returncode == 0, result.stderr
        assert 'NEXUTERM_SMOKE_VNC_OK' in result.stderr, result.stderr
        if args.password:
            assert ('password-response',) in events
        for event in [('frame',), ('key', 1, ord('a')), ('key', 0, ord('a')),
                      ('pointer', 0, 32, 24), ('pointer', 8, 32, 24), ('closed',)]:
            assert event in events, (event, events)
        print('PASS: VNC integrado, framebuffer real, teclado, ratón, rueda y cierre de conexión')


if __name__ == '__main__':
    main()
