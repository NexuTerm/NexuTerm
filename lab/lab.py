#!/usr/bin/env python3
"""Laboratorio propio y local; nunca usa equipos ni credenciales personales."""
import argparse, json, os, pathlib, secrets, socket, subprocess, sys

ROOT=pathlib.Path(__file__).resolve().parent.parent
DATA=ROOT/'.lab'
NAME='nexuterm-lab-'+str(os.getuid())
IMAGE='localhost/nexuterm-lab:ubuntu24'
PORTS=(22222,13389,15901)

def run(*args, **kw):
    return subprocess.run(args, check=True, **kw)

def up():
    for port in PORTS:
        with socket.socket() as s:
            s.bind(('127.0.0.1',port))
    DATA.mkdir(mode=0o700, exist_ok=True)
    if not (DATA/'password').exists():
        fd=os.open(DATA/'password',os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
        with os.fdopen(fd,'w') as f:f.write(secrets.token_hex(12))
    if not (DATA/'id_ed25519').exists():
        run('ssh-keygen','-q','-t','ed25519','-N','','-f',str(DATA/'id_ed25519'))
    run('podman','build','-t',IMAGE,str(ROOT/'lab'))
    run('podman','run','--detach','--name',NAME,'--network=slirp4netns',
        '-p','127.0.0.1:22222:22','-p','127.0.0.1:13389:3389','-p','127.0.0.1:15901:5901',
        '-v',str(DATA)+':/run/lab:ro',IMAGE,stdout=subprocess.DEVNULL)
    print('Laboratorio iniciado. SSH localhost:22222; RDP localhost:13389; VNC localhost:15901.')
    print('Usuario: tester. Clave y contraseña ficticias en .lab/, excluido de Git.')

def trust():
    result=run('podman','exec',NAME,'cat','/etc/ssh/ssh_host_ed25519_key.pub',capture_output=True,text=True)
    (DATA/'known_hosts').write_text('[localhost]:22222,[127.0.0.1]:22222 '+result.stdout)
    cert=run('podman','exec',NAME,'cat','/etc/xrdp/cert.pem',capture_output=True)
    (DATA/'rdp-ca.pem').write_bytes(cert.stdout)
    print('Claves de servidor copiadas directamente del contenedor propio.')

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['up','trust','down','status']);a=p.parse_args()
    if a.action=='up':up()
    elif a.action=='trust':trust()
    elif a.action=='down':run('podman','rm','--force',NAME)
    else:run('podman','inspect','--format','{{.State.Status}}',NAME)
