#!/usr/bin/env bash
set -e

function get_arch() {
    a=$(uname -m)
    case ${a} in
    "x86_64" | "amd64" | "x86-64")
        echo "x86_64"
        ;;
    "i386" | "i486" | "i586")
        echo "386"
        ;;
    "aarch64" | "arm64")
        echo "aarch64"
        ;;
    "armv6l" | "armv7l")
        echo "arm"
	;;
    *)
        echo ${NIL}
        ;;
    esac
}

function get_os() {
    echo $(uname -s | awk '{print tolower($0)}')
}

main() {
    local abi=$1
    local release="0.1.0"
    local os=$(get_os)
    local arch=$(get_arch)
    local dest_file="${HOME}/.sys-kaleido/downloads/sys-kaleido${release}.${os}-${arch}"
    local url="https://github.com/jinyuli/sys-kaleido/releases/download/v${release}/sys-kaleido-${arch}-unknown-${os}-${abi}"
    local get_return=0

    if [[ "$os" == "darwin" ]]; then
        url="https://github.com/jinyuli/sys-kaleido/releases/download/v${release}/sys-kaleido-${arch}-apple-${os}"
    fi

    echo "[1/3] Downloading ${url}"
    rm -f "${dest_file}"
    mkdir -p "${HOME}/.sys-kaleido/downloads"
    if [ -x "$(command -v wget)" ]; then
        wget -q --show-progress -O "${dest_file}" "${url}" || get_return=$?
    else
        echo "use curl"
        curl -s -S -L --create-dirs -o "${dest_file}" "${url}" || get_return=$?
    fi

    echo "return: ${get_return}"
    if [[ $get_return -ne 0 ]]; then
        echo "failed to download file, you may try to download source code and build sys-kaleido manually"
        return
    fi

    echo "[2/3] Install sys-kaleido to the ${HOME}/.sys-kaleido/bin"
    mkdir -p "${HOME}/.sys-kaleido/bin"
    cp "${dest_file}" "${HOME}/.sys-kaleido/bin/sys-kaleido"
    chmod +x "${HOME}/.sys-kaleido/bin/sys-kaleido"

    echo "[3/3] Set environment variables"
    cat >${HOME}/.sys-kaleido/env <<-'EOF'
#!/bin/sh
# sys-kaleido shell setup
export PATH="${HOME}/.sys-kaleido/bin:$PATH"
	EOF


    if [ -x "$(command -v bash)" ]; then
        cat >>${HOME}/.bashrc <<-'EOF'
if [ -f "${HOME}/.sys-kaleido/env" ]; then
    . "${HOME}/.sys-kaleido/env"
fi
		EOF
    fi

    if [ -x "$(command -v zsh)" ]; then
        cat >>${HOME}/.zshrc <<-'EOF'
if [ -f "${HOME}/.sys-kaleido/env" ]; then
    . "${HOME}/.sys-kaleido/env"
fi
		EOF
    fi

    echo "sys-kaleido $release installed!"
    echo -e "\nTo configure your current shell, run:\nsource \"$HOME/.sys-kaleido/env\""

    exit 0
}

options=$(getopt -o A: -l abi: -- "$@")
if [[$? -eq 0]]; then
    echo "invalid options provided"
    exit 1
fi
eval set -- "$options"
while true; do
    echo "option: $1"
    case "$1" in
        -A | --abi)
            shift;
            abi=$1
            ;;
        --)
            shift
            break
            ;;
    esac
    shift
done

if [[ "$abi" == "" ]]; then
    abi="gnu"
fi

main "$abi"
