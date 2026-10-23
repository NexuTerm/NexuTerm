#!/usr/bin/env python3
"""Validate runtime tools and build a Debian/Ubuntu package with dependencies."""
import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parent.parent
CLIENTS = (
    ("SSH/SFTP", ("ssh",), "openssh-client"),
    ("Telnet", ("telnet",), "inetutils-telnet"),
    ("FTP Windows CE", ("curl",), "curl"),
)
MINIMUM_LIBRARIES = {
    "libgtk-4-1": "4.10",
    "libadwaita-1-0": "1.4",
    "libvte-2.91-gtk4-0": "0.70",
}


def output(*args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def find_client(names):
    # Match the application's refusal to execute clients from relative PATH entries.
    for name in names:
        for directory in os.get_exec_path():
            path = Path(directory) / name
            if path.is_absolute() and path.is_file() and os.access(path, os.X_OK):
                return path
    return None


def find_ftp_backend():
    candidates = [Path("/usr/libexec/gvfsd-ftp"), Path("/usr/lib/gvfs/gvfsd-ftp")]
    candidates.extend(Path("/usr/lib").glob("*/gvfs/gvfsd-ftp"))
    return next((path for path in candidates if path.is_file() and os.access(path, os.X_OK)), None)


def check_runtime(binary):
    missing = [f"{protocol}: instala {package.replace(' | ', ' o ')}"
               for protocol, names, package in CLIENTS
               if find_client(names) is None]
    if find_ftp_backend() is None:
        missing.append("FTP gráfico: instala gvfs-backends")
    if not binary.is_file():
        missing.append(f"No existe el ejecutable: {binary}")
    else:
        linked = output("ldd", str(binary.resolve()))
        missing.extend(line.strip() for line in linked.splitlines() if "not found" in line)
    if missing:
        raise RuntimeError("Dependencias pendientes:\n- " + "\n- ".join(missing)
                           + "\nInstala el .deb con sudo apt install ./dist/nexuterm_*.deb"
                           + " para resolver las dependencias automáticamente.")
    print("Dependencias de ejecución correctas: SSH/SFTP, Telnet, FTP, RDP y bibliotecas enlazadas (FTP gráfico, RDP y VNC están integrados).")


def package_dependencies(detected):
    libraries = detected.split(", ") if detected else []
    for package, minimum in MINIMUM_LIBRARIES.items():
        for index, dependency in enumerate(libraries):
            match = re.fullmatch(re.escape(package) + r" \(>= ([^)]+)\)", dependency)
            if match:
                if subprocess.run(["dpkg", "--compare-versions", match[1], "lt", minimum],
                                  check=False).returncode == 0:
                    libraries[index] = f"{package} (>= {minimum})"
                break
        else:
            libraries.append(f"{package} (>= {minimum})")
    return ", ".join(libraries + [package for _, _, package in CLIENTS]
                     + ["ca-certificates", "dbus-user-session", "gnome-keyring | keepassxc", "gvfs-backends"])


def build_deb(binary, destination):
    binary = binary.resolve(strict=True)
    for command in ("dpkg-deb", "dpkg-shlibdeps", "dpkg"):
        if not shutil.which(command):
            raise RuntimeError("Instala dpkg-dev y binutils para preparar el release.")
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
    architecture = output("dpkg", "--print-architecture")
    destination.mkdir(parents=True, exist_ok=True)
    artifact = destination.resolve() / f"nexuterm_{version}_{architecture}.deb"
    with tempfile.TemporaryDirectory(prefix="nexuterm-deb-") as directory:
        source = Path(directory)
        package = source / "debian/nexuterm"
        for original, relative, mode in (
            (binary, "usr/bin/nexuterm", 0o755),
            (ROOT / "packaging/io.nexuterm.NexuTerm.desktop",
             "usr/share/applications/io.nexuterm.NexuTerm.desktop", 0o644),
            (ROOT / "logo/NexuTerm.png", "usr/share/pixmaps/io.nexuterm.NexuTerm.png", 0o644),
        ):
            target = package / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(original, target)
            target.chmod(mode)
        # dpkg-shlibdeps needs source-package metadata; analyze the staged ELF.
        (source / "debian/control").write_text(
            "Source: nexuterm\n\nPackage: nexuterm\nArchitecture: any\n"
            "Description: Terminal y conexiones remotas\n")
        (package / "DEBIAN").mkdir()
        detected = output("dpkg-shlibdeps", "-O", "-edebian/nexuterm/usr/bin/nexuterm",
                          cwd=source)
        fields = dict(line.split("=", 1) for line in detected.splitlines() if "=" in line)
        if not fields.get("shlibs:Depends"):
            raise RuntimeError("No se pudieron determinar las bibliotecas del ejecutable.")
        control = package / "DEBIAN/control"
        control.write_text(
            f"Package: nexuterm\nVersion: {version}\nArchitecture: {architecture}\n"
            "Maintainer: NexuTerm contributors\nSection: net\nPriority: optional\n"
            f"Depends: {package_dependencies(fields['shlibs:Depends'])}\n"
            "Homepage: https://github.com/NexuTerm/NexuTerm\n"
            "Description: Terminal y conexiones remotas para Linux\n"
            " Terminal GTK con conexiones SSH, SFTP, RDP y VNC.\n")
        package.chmod(0o755)
        for path in package.rglob("*"):
            if path.is_dir():
                path.chmod(0o755)
        subprocess.run(["dpkg-deb", "--root-owner-group", "--build", str(package),
                        str(artifact)], check=True)
    print(f"Release: {artifact}")
    print(f"Instalar: sudo apt install {artifact}")
    return artifact


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("check", "deb"))
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/nexuterm")
    parser.add_argument("--output-dir", type=Path, default=ROOT / "dist")
    args = parser.parse_args()
    try:
        if args.action == "check":
            check_runtime(args.binary)
        else:
            build_deb(args.binary, args.output_dir)
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        parser.exit(1, f"Error: {error}\n")


if __name__ == "__main__":
    main()
