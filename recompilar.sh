#!/usr/bin/env bash
set -Eeuo pipefail

usage() {
    printf 'Uso: %s -r | -d\n  -r  Recompilar release\n  -d  Recompilar debug\n' "${0##*/}"
}

if [[ $# -eq 1 && "$1" == '-h' ]]; then
    usage
    exit 0
fi
if [[ $# -ne 1 ]]; then
    usage >&2
    exit 2
fi

build_args=(build --locked --jobs "${CARGO_BUILD_JOBS:-2}")
case "$1" in
    -r) build_args+=(--release); profile='release' ;;
    -d) profile='debug' ;;
    *) usage >&2; exit 2 ;;
esac

project_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
cargo_command="${CARGO:-cargo}"
# Reuse an installed toolchain without changing the user's global configuration.
if [[ -z "${CARGO:-}" ]] && ! command -v "$cargo_command" >/dev/null 2>&1; then
    if [[ -x "$HOME/.cargo/bin/cargo" ]]; then
        cargo_command="$HOME/.cargo/bin/cargo"
    elif [[ -x "$HOME/.local/share/rea/toolchains/cargo/bin/cargo" ]]; then
        export CARGO_HOME="${CARGO_HOME:-$HOME/.local/share/rea/toolchains/cargo}"
        export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.local/share/rea/toolchains/rustup}"
        cargo_command="$HOME/.local/share/rea/toolchains/cargo/bin/cargo"
    fi
fi
if ! command -v "$cargo_command" >/dev/null 2>&1; then
    printf 'No se encuentra Cargo. Activa el toolchain de Rust en PATH o indica su ruta mediante CARGO.\n' >&2
    exit 127
fi

if [[ "$cargo_command" == */* ]]; then
    export PATH="$(dirname -- "$cargo_command"):$PATH"
fi

cd -- "$project_root"
printf 'Recompilando NexuTerm (%s)…\n' "$profile"
"$cargo_command" "${build_args[@]}"
printf 'Compilación %s completada.\n' "$profile"
