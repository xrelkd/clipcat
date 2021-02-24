# Clipcat

[![CI](https://github.com/xrelkd/clipcat/workflows/Build/badge.svg)](https://github.com/xrelkd/clipcat/actions)

`Clipcat` is a clipboard manager written in [Rust Programming Language](https://www.rust-lang.org/).

## Architecture

Clipcat uses the Client-Server architecture. There are two role types in this architecture: `Server` and `Client`.

### Clipcat Server

A `clipcat` server (as known as daemon) is running as the background process that doing the following routines:

- Watching the change for `X11 clipboard`.
- Caching the content of `X11 clipboard`.
- Insert content into `X11 clipboard`.
- Serving as a `gRPC` server and waiting for remote procedure call from clients.

### Clipcat Client

A `clipcat` client sends requests to the server for the following manipulations:

- List: list the cached clips from server.
- Insert: replace the current content of `X11 clipboard` with a clip.
- Remove: remove the cached clips from server.

### List of Implementations

| Program        | Role Type | Comment                                                                                |
| -------------- | --------- | -------------------------------------------------------------------------------------- |
| `clipcatd`     | `Server`  | The `clipcat` server (daemon)                                                          |
| `clipcatctl`   | `Client`  | The `clipcat` client which provides a command line interface                           |
| `clipcat-menu` | `Client`  | The `clipcat` client which calls built-in finder or external finder for selecting clip |

## Quick Start

### Installation

| Linux Distribution                  | Package Manager                     | Package                                                                                             | Command                       |
| ----------------------------------- | ----------------------------------- | --------------------------------------------------------------------------------------------------- | ----------------------------- |
| Various                             | [Nix](https://github.com/NixOS/nix) | [clipcat](https://github.com/xrelkd/nixpkgs/blob/master/pkgs/applications/misc/clipcat/default.nix) | `nix-env -iA nixpkgs.clipcat` |
| [NixOS](https://nixos.org)          | [Nix](https://github.com/NixOS/nix) | [clipcat](https://github.com/xrelkd/nixpkgs/blob/master/pkgs/applications/misc/clipcat/default.nix) | `nix-env -iA nixos.clipcat`   |
| [Arch Linux](https://archlinux.org) | [Yay](https://github.com/Jguer/yay) | [clipcat](https://aur.archlinux.org/packages/clipcat/)                                              | `yay -S clipcat`              |

### Usage

0. Setup configurations for `clipcat`.

```console
$ mkdir -p                       $XDG_CONFIG_HOME/clipcat
$ clipcatd default-config      > $XDG_CONFIG_HOME/clipcat/clipcatd.toml
$ clipcatctl default-config    > $XDG_CONFIG_HOME/clipcat/clipcatctl.toml
$ clipcat-menu default-config  > $XDG_CONFIG_HOME/clipcat/clipcat-menu.toml
```

1. Start `clipcatd` for watching clipboard events.

```console
$ clipcatd
```

2. Copy arbitrary text from other X11 process with your mouse or keyboard.

3. You can do one of the following manipulations with `clipcatctl` or `clipcat-menu`:

| Command                   | Comment                                           |
| ------------------------- | ------------------------------------------------- |
| `clipcatctl list`         | List cached clipboard history                     |
| `clipcatctl promote <id>` | Insert cached clip with `<id>` into X11 clipboard |
| `clipcatctl remove [ids]` | Remove cahced clips with `[ids]` from server      |
| `clipcatctl clear`        | Clear cached clipboard history                    |

| Command               | Comment                                 |
| --------------------- | --------------------------------------- |
| `clipcat-menu insert` | Insert a cached clip into X11 clipboard |
| `clipcat-menu remove` | Remove cached clips from server         |
| `clipcat-menu edit`   | Edit a cached clip with `\$EDITOR`      |

**Note**: Supported finders for `clipcat-menu`:

- built-in finder (integrate with crate [skim](https://github.com/lotabout/skim))
- [skim](https://github.com/lotabout/skim)
- [fzf](https://github.com/junegunn/fzf)
- [rofi](https://github.com/davatorium/rofi)
- [dmenu](https://tools.suckless.org/dmenu/)

### Configuration

| Program        | Default Configuration File Path              |
| -------------- | -------------------------------------------- |
| `clipcatd`     | `$XDG_CONFIG_HOME/clipcat/clipcatd.toml`     |
| `clipcatctl`   | `$XDG_CONFIG_HOME/clipcat/clipcatctl.toml`   |
| `clipcat-menu` | `$XDG_CONFIG_HOME/clipcat/clipcat-menu.toml` |

#### Configuration for `clipcatd`

```toml
daemonize = true          # run as a traditional UNIX daemon
max_history = 50          # max clip history limit
log_level = 'INFO'        # log level

[watcher]
load_current = true       # load current clipboard content at startup
enable_clipboard = true   # watch X11 clipboard
enable_primary = true     # watch X11 primary clipboard

[grpc]
host = '127.0.0.1'        # host address for gRPC
port = 45045              # port number for gRPC
```

#### Configuration for `clipcatctl`

```toml
server_host = '127.0.0.1' # host address of clipcat gRPC server
server_port = 45045       # port number of clipcat gRPC server
log_level = 'INFO'        # log level
```

#### Configuration for `clipcat-menu`

```toml
server_host = '127.0.0.1' # host address of clipcat gRPC server
server_port = 45045       # port number of clipcat gRPC server
finder = 'rofi'           # the default finder to invoke when no "--finder=<finder>" option provided

[rofi]                    # options for "rofi"
line_length = 100         # length of line
menu_length = 30          # length of menu

[dmenu]                   # options for "dmenu"
line_length = 100         # length of line
menu_length = 30          # length of menu

[custom_finder]           # customize your finder
program = 'fzf'           # external program name
args = []                 # arguments for calling external program
```

## Integration

### Integrating with [Zsh](https://www.zsh.org/)

For `zsh` user, it will be useful to integrate `clipcat` with `zsh`.

Add the following command in your `zsh` configuration file (`~/.zshrc`):

```bash
if type clipcat-menu >/dev/null 2>&1; then
    alias clipedit=' clipcat-menu --finder=builtin edit'
    alias clipdel=' clipcat-menu --finder=builtin remove'

    bindkey -s '^\' "^Q clipcat-menu --finder=builtin insert ^J"
    bindkey -s '^]' "^Q clipcat-menu --finder=builtin remove ^J"
fi
```

### Integrating with [i3 Window Manager](https://i3wm.org/)

For `i3` window manager user, it will be useful to integrate `clipcat` with `i3`.

Add the following options in your `i3` configuration file (`$XDG_CONFIG_HOME/i3/config`):

```
exec_always --no-startup-id clipcatd                # start clipcatd at startup

set $launcher-clipboard-insert clipcat-menu insert
set $launcher-clipboard-remove clipcat-menu remove

bindsym $mod+p exec $launcher-clipboard-insert
bindsym $mod+o exec $launcher-clipboard-remove
```

**Note**: You can use `rofi` or `dmenu` as the default finder.

## Compiling from Source

`clipcat` requires the following tools and packages to build:

- `git`
- `rustc`
- `cargo`
- `pkgconfig`
- `protobuf`
- `clang`
- `libclang`
- `libxcb`

With the above tools and packages already installed, you can simply run:

```console
$ git clone https://github.com/xrelkd/clipcat.git
$ cd clipcat
$ cargo build --release --features=all
```

## License

Clipcat is licensed under the GNU General Public License version 3. See [LICENSE](./LICENSE) for more information.
