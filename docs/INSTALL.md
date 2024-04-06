# Installation

- Table of contents

  - [Install the pre-built binaries](#install-the-pre-built-binaries)
  - [Build from source](#build-from-source)
  - [Install via Package Manager](#install-via-package-manager)
    - [NixOS and Nix](#nixos-and-nix)
    - [Arch Linux](#arch-linux)
    - [Debian and ubuntu derivatives](#debian-and-ubuntu-derivatives)
    - [Fedora Linux](#fedora-linux)
    - [FreeBSD](#freebsd)

## Install the pre-built binaries

Pre-built binaries for Linux can be found on [the releases page](https://github.com/xrelkd/clipcat/releases/), the latest release is available [here](https://github.com/xrelkd/clipcat/releases/latest).

For example, to install `clipcat` to `~/bin`:

```bash
# create ~/bin
mkdir -p ~/bin

# change directory to ~/bin
cd ~/bin

# download and extract clipcat to ~/bin/
# NOTE: you can replace the version with the version you want to install
export CLIPCAT_VERSION=$(basename $(curl -s -w %{redirect_url} https://github.com/xrelkd/clipcat/releases/latest))

# NOTE: the architecture of your machine,
# available values are `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`
export ARCH=x86_64-unknown-linux-musl
curl -s -L "https://github.com/xrelkd/clipcat/releases/download/${CLIPCAT_VERSION}/clipcat-${CLIPCAT_VERSION}-${ARCH}.tar.gz" | tar xzf -

# add `~/bin` to the paths that your shell searches for executables
# this line should be added to your shells initialization file,
# e.g. `~/.bashrc` or `~/.zshrc`
export PATH="$PATH:$HOME/bin"

# show version info
clipcatd     version
clipcatctl   version --client
clipcat-menu version --client
```

## Build from source

`clipcat` requires the following tools and packages to build:

- `rustc`
- `cargo`
- `protobuf-compiler`

With the above tools and packages already installed, you can simply run:

```bash
git clone --branch=main https://github.com/xrelkd/clipcat.git
cd clipcat

cargo install --path clipcatd
cargo install --path clipcatctl
cargo install --path clipcat-menu
```

## Install via Package Manager

### NixOS and Nix

#### Install via `nix profile`

```bash
nix profile install 'github:xrelkd/clipcat/main'
```

#### Install on `NixOS`

```bash
nix-env -iA nixos.clipcat
```

#### Install via `Nix` on various Linux distribution

```bash
nix-env -iA nixpkgs.clipcat
```

### Arch Linux

```bash
pacman -S clipcat
```

### Debian and Ubuntu derivatives

```bash
export CLIPCAT_VERSION=$(basename $(curl -s -w %{redirect_url} https://github.com/xrelkd/clipcat/releases/latest))

curl -s -L -O https://github.com/xrelkd/clipcat/releases/download/${CLIPCAT_VERSION}/clipcat_${CLIPCAT_VERSION#v}_amd64.deb
dpkg -i clipcat_${CLIPCAT_VERSION#v}_amd64.deb
```

### Fedora Linux

```bash
export CLIPCAT_VERSION=$(basename $(curl -s -w %{redirect_url} https://github.com/xrelkd/clipcat/releases/latest))

curl -s -L -O https://github.com/xrelkd/clipcat/releases/download/${CLIPCAT_VERSION}/clipcat-${CLIPCAT_VERSION#v}-1.el7.x86_64.rpm
dnf install --assumeyes clipcat-${CLIPCAT_VERSION#v}-1.el7.x86_64.rpm
```

### FreeBSD

```bash
pkg install clipcat
```
