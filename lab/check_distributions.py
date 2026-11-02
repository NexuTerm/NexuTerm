#!/usr/bin/env python3
"""Native isolated acceptance builds. Uses Podman; never mounts credentials or host config."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile
import time
import uuid
ROOT=Path(__file__).resolve().parent.parent
PACKAGES=['build-essential','pkg-config','libglib2.0-dev-bin','libgtk-4-dev','libvte-2.91-gtk4-dev','libadwaita-1-dev','libvncserver-dev','libsecret-1-dev','libssl-dev','openssh-client','openssh-server','inetutils-telnet','curl','gvfs-backends','dbus-user-session','dbus-x11','gnome-keyring','python3','xvfb','xauth','fonts-dejavu-core','adwaita-icon-theme','dpkg-dev','binutils']

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain',type=Path,required=True)
    parser.add_argument('--registry',type=Path,required=True)
    parser.add_argument('--output-dir',type=Path,required=True)
    parser.add_argument('--image',action='append',choices=['ubuntu:24.04','debian:13'])
    parser.add_argument('--keep-failed',action='store_true',help='retain only a failed disposable container for diagnostics')
    parser.add_argument('--resume',action='store_true',help='reuse previous native target caches with a new source snapshot')
    parser.add_argument('--release',action='store_true',help='also compile/package an optimized binary')
    args=parser.parse_args()
    toolchain=args.toolchain.resolve(strict=True)
    registry=args.registry.resolve(strict=True)
    if not (toolchain/'bin/rustc').is_file() or not (toolchain/'bin/cargo').is_file() or not registry.is_dir():
        parser.error('Expected an installed Rust toolchain and cached Cargo registry')
    output=args.output_dir.resolve()
    output.mkdir(parents=True,exist_ok=True)
    # Snapshot tracked and explicitly unignored sources, including this work in progress.
    paths=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','--','Cargo.toml','Cargo.lock','build.rs','README.md','src','tests','lab','packaging','logo'],cwd=ROOT,text=True).splitlines()
    if args.resume and not (output/'matrix.json').is_file():
        parser.error('--resume requires a previous matrix.json in the output directory')
    source=output/('source-'+uuid.uuid4().hex[:8])
    source.mkdir(exist_ok=False)
    for relative in paths:
        original=ROOT/relative
        if not original.is_file() or original.is_symlink():
            continue
        target=source/relative
        target.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(original,target)
        target.chmod(original.stat().st_mode & 0o777)
    (source/'target').symlink_to('/target',target_is_directory=True)
    (source/'artifacts').mkdir()
    digest=hashlib.sha256()
    for path in sorted(p for p in source.rglob('*') if p.is_file() and not p.is_symlink()):
        digest.update(str(path.relative_to(source)).encode());digest.update(path.read_bytes())
    source_digest=digest.hexdigest()
    reports=[]
    for image in args.image or ['ubuntu:24.04','debian:13']:
        tag=image.replace(':','-').replace('.','_')
        folder=output/tag
        folder.mkdir(exist_ok=args.resume)
        (folder/'target').mkdir(exist_ok=args.resume)
        (folder/'artifacts').mkdir(exist_ok=args.resume)
        name='nexuterm-acceptance-'+uuid.uuid4().hex[:12]
        record={'image':image,'container':name,'steps':[],'source_snapshot':str(source),'source_sha256':source_digest}
        log_path=folder/'acceptance.log'
        def run(label,command,timeout=1800):
            print(f'{tag}: {label}',flush=True)
            started=time.monotonic()
            with log_path.open('a') as log:
                log.write('\nSTEP '+label+'\n')
                log.flush()
                result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=timeout)
            record['steps'].append(dict(name=label,returncode=result.returncode,seconds=round(time.monotonic()-started,3)))
            (folder/'result.json').write_text(json.dumps(record,indent=2))
            result.check_returncode()
        full_image='docker.io/library/'+image
        run('pull image',['podman','pull',full_image],300)
        record['image_id']=subprocess.check_output(['podman','image','inspect',full_image,'--format','{{.Id}}'],text=True).strip()
        started=False
        try:
            run('start isolated container',['podman','run','-d','--name',name,'--userns','keep-id','--user','0','--cpus','2','--memory','4g','--network','slirp4netns',
                '-v',f'{source}:/work:ro','-v',f'{folder}/target:/target:rw','-v',f'{folder}/artifacts:/work/artifacts:rw',
                '-v',f'{toolchain}:/opt/rust:ro','-v',f'{registry}:/cargo/registry:ro',
                '-e','PATH=/opt/rust/bin:/usr/sbin:/usr/bin:/sbin:/bin','-e','CARGO_HOME=/cargo','-e','CARGO_TARGET_DIR=/target','-e','CARGO_BUILD_JOBS=1','-w','/work',full_image,'sleep','infinity'],120)
            started=True
            run('install build and test prerequisites',['podman','exec','-e','DEBIAN_FRONTEND=noninteractive',name,'/bin/sh','-c',
                'apt-get update && apt-get install -y --no-install-recommends '+shlex.join(PACKAGES)+' && mkdir -p /run/sshd'],900)
            run('create unprivileged test account',['podman','exec',name,'useradd','--non-unique','--uid',str(os.getuid()),'--create-home','--shell','/bin/sh','nexuterm-test'])
            # Nonlocked account with an unusable password hash; sshd accepts only fixture keys.
            run('enable key-only fixture account',['podman','exec',name,'usermod','--password','x','nexuterm-test'])
            run('prepare writable Cargo metadata',['podman','exec',name,'chown',str(os.getuid()),'/cargo'])
            test_exec=['podman','exec','--user','nexuterm-test','-e','HOME=/home/nexuterm-test','-e','USER=nexuterm-test','-e','LOGNAME=nexuterm-test',name]
            run('record native versions',['podman','exec',name,'/bin/sh','-c','cat /etc/os-release; rustc --version; pkg-config --modversion gtk4 libadwaita-1 vte-2.91-gtk4; getconf GNU_LIBC_VERSION'])
            run('native full test suite',test_exec+['cargo','test','--locked','--offline','--jobs','1'],2400)
            run('native graphical binary',test_exec+['cargo','build','--locked','--offline','--jobs','1'],1200)
            for script in ['check_interventions.py','check_recipes.py','check_supervised.py','check_remote_editor.py']:
                run(script,test_exec+['python3','lab/'+script],600)
            profile='debug'
            if args.release:
                run('native optimized binary',test_exec+['cargo','build','--locked','--offline','--release','--jobs','1'],2400)
                profile='release'
            run('runtime dependency check',test_exec+['python3','packaging/release.py','check','--binary',f'/target/{profile}/nexuterm'])
            run('Debian package creation',test_exec+['python3','packaging/release.py','deb','--binary',f'/target/{profile}/nexuterm','--output-dir','/work/artifacts/package'])
            record['success']=True
        finally:
            # Delete only the unique disposable container created by this invocation.
            exists=subprocess.run(['podman','container','exists',name],check=False,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode==0
            keep=args.keep_failed and not record.get('success',False) and exists
            if keep:
                record['retained_for_diagnostics']=name
                print(f'Retained failed disposable container: {name}',flush=True)
            if (started or exists) and not keep:
                subprocess.run(['podman','rm','-f',name],check=False,stdout=subprocess.DEVNULL)
            record['finished_at']=int(time.time())
            (folder/'result.json').write_text(json.dumps(record,indent=2))
            reports.append(record)
            (output/'matrix.json').write_text(json.dumps(reports,indent=2))
        print(f'PASS {tag}: native tests, GUI/SSH recovery and package',flush=True)

if __name__=='__main__':
    main()
