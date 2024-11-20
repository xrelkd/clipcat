<h1 align="center">Clipcat</h1>

<p align="center">
    A clipboard manager written in
    <a href="https://www.rust-lang.org/" target="_blank">Rust Programming Language</a>.
</p>

<p align="center">
    <a href="https://github.com/xrelkd/clipcat/releases"><img src="https://img.shields.io/github/v/release/xrelkd/clipcat.svg"></a>
    <a href="https://deps.rs/repo/github/xrelkd/clipcat"><img src="https://deps.rs/repo/github/xrelkd/clipcat/status.svg"></a>
    <a href="https://github.com/xrelkd/clipcat/actions?query=workflow%3ARust"><img src="https://github.com/xrelkd/clipcat/workflows/Rust/badge.svg"></a>
    <a href="https://github.com/xrelkd/clipcat/actions?query=workflow%3ARelease"><img src="https://github.com/xrelkd/clipcat/workflows/Release/badge.svg"></a>
    <a href="https://github.com/xrelkd/clipcat/blob/main/LICENSE"><img alt="GitHub License" src="https://img.shields.io/github/license/xrelkd/clipcat"></a>
</p>

**[Installation](#installation) | [Usage](#usage) | [Integration](#integration)**

<details>
<summary>Table of contents</summary>

- [Features](#features)
- [Installation](#installation)
- [Architecture](#architecture)
- [Usage](#usage)
- [Configuration](#configuration)
- [Integration](#integration)
- [Programs in this Repository](#programs-in-this-repository)
- [License](#license)

</details>

## Features

- [x] Copy/Paste plaintext
- [x] Copy/Paste images
- [x] Persistent clipboard contents
- [x] Support for snippets
- [x] Support for `X11`
- [ ] ~~Support for `Wayland` (experimental)~~
- [x] Support for `macOS`
- [x] Support for `gRPC`
  - [x] gRPC over `HTTP`
  - [x] gRPC over `Unix domain socket`
- [x] Support for `D-Bus`

## Screenshots and Demonstration

- Demonstration with [Rofi](https://github.com/davatorium/rofi)

  https://github.com/xrelkd/clipcat/assets/46590321/606a6a3a-6d7d-49d1-98c7-988e3d72df30

- Use [Rofi](https://github.com/davatorium/rofi) to select clip

  ![screenshot finder rofi](docs/_static/screenshot-finder-rofi.png)

- Use [dmenu](https://tools.suckless.org/dmenu/) to select clip

  ![screenshot finder dmenu](docs/_static/screenshot-finder-dmenu.png)

- Use [skim](https://github.com/lotabout/skim) to select clip

  ![screenshot finder skim](docs/_static/screenshot-finder-skim.png)

## Installation

`Clipcat` can be installed using various package managers on Linux.

Pre-built binaries can also be downloaded from the [GitHub releases page](https://github.com/xrelkd/clipcat/releases).

Detailed instructions for installing `Clipcat` can be found [here](docs/INSTALL.md).

## Architecture

Clipcat uses a Client-Server architecture. There are two role types in this architecture: `Server` and `Client`.

### Clipcat Server

The `clipcat` daemon runs as a background process and performs the following tasks:

- Watches for changes to the clipboard.
- Caches clipboard content.
- Inserts content into the clipboard.
- Acts as a gRPC server, waiting for remote procedure calls from clients.

Currently, `clipcat` supports the following [windowing systems](https://en.wikipedia.org/wiki/Windowing_system):

- `X11`, the following `crate`s are leveraged:
  - [x11rb](https://github.com/psychon/x11rb)
  - [arboard](https://github.com/1Password/arboard)
- ~~`Wayland` (experimentally), the following `crate`s are leveraged:~~
  - ~~[wl-clipboard-rs](https://github.com/YaLTeR/wl-clipboard-rs)~~
  - ~~[arboard](https://github.com/1Password/arboard)~~

### Clipcat Client

A `clipcat` client sends requests to the server for the following operations:

- `List`: list the cached clips from server.
- `Insert`: replace the current content of `clipboard` with a clip.
- `Remove`: remove the cached clips from server.

### List of Implementations

| Program        | Role Type | Comment                                                                                      |
| -------------- | --------- | -------------------------------------------------------------------------------------------- |
| `clipcatd`     | `Server`  | The `clipcat` server (daemon).                                                               |
| `clipcatctl`   | `Client`  | The `clipcat` client providing a command line interface.                                     |
| `clipcat-menu` | `Client`  | The `clipcat` client that calls a built-in finder or an external finder for selecting clips. |

## Usage

0. Setup configurations for `clipcat`. Read [configuration](#configuration) section for more details.

```bash
mkdir -p                       $XDG_CONFIG_HOME/clipcat
clipcatd default-config      > $XDG_CONFIG_HOME/clipcat/clipcatd.toml
clipcatctl default-config    > $XDG_CONFIG_HOME/clipcat/clipcatctl.toml
clipcat-menu default-config  > $XDG_CONFIG_HOME/clipcat/clipcat-menu.toml
```

1. Start `clipcatd` for watching clipboard events.

```bash
# Show the usage. Please read the usage before doing any other operations.
clipcatd help

# Start and daemonize clipcatd. It will run in the background.
# You can use `pkill clipcatd` to stop it; a `SIGTERM` signal will be sent to clipcatd.
clipcatd

# Alternatively, you can start clipcatd but keep it in the foreground.
# You can press `Ctrl+C` in your terminal to stop it; a `SIGINT` signal will be sent to clipcatd.
clipcatd --no-daemon
```

2. Copy arbitrary text or images from other processes using your mouse or keyboard.

3. You can run the following commands with `clipcatctl` or `clipcat-menu`:

| Command                   | Comment                                               |
| ------------------------- | ----------------------------------------------------- |
| `clipcatctl list`         | List cached clipboard history                         |
| `clipcatctl promote <id>` | Insert cached clip with `<id>` into the X11 clipboard |
| `clipcatctl remove [ids]` | Remove cached clips with `[ids]` from the server      |
| `clipcatctl clear`        | Clear cached clipboard history                        |

| Command               | Comment                                     |
| --------------------- | ------------------------------------------- |
| `clipcat-menu insert` | Insert a cached clip into the X11 clipboard |
| `clipcat-menu remove` | Remove cached clips from the server         |
| `clipcat-menu edit`   | Edit a cached clip with `$EDITOR`           |

The following finders are supported by `clipcat-menu`:

- Built-in finder (integrating with the crate [skim](https://github.com/lotabout/skim))
- [skim](https://github.com/lotabout/skim)
- [fzf](https://github.com/junegunn/fzf)
- [rofi](https://github.com/davatorium/rofi)
- [dmenu](https://tools.suckless.org/dmenu/)

## Configuration

| Program        | Default Configuration File Path              |
| -------------- | -------------------------------------------- |
| `clipcatd`     | `$XDG_CONFIG_HOME/clipcat/clipcatd.toml`     |
| `clipcatctl`   | `$XDG_CONFIG_HOME/clipcat/clipcatctl.toml`   |
| `clipcat-menu` | `$XDG_CONFIG_HOME/clipcat/clipcat-menu.toml` |

<details>
    <summary>Configuration for <b>clipcatd</b></summary>

```toml
# Run as a traditional UNIX daemon.
daemonize = true

# Maximum number of clips in history.
max_history = 50

# File path for clip history.
# If this value is omitted, `clipcatd` will persist history in `$XDG_CACHE_HOME/clipcat/clipcatd-history`.
history_file_path = "/home/<username>/.cache/clipcat/clipcatd-history"

# File path for the PID file.
# If this value is omitted, `clipcatd` will place the PID file in `$XDG_RUNTIME_DIR/clipcatd.pid`.
pid_file = "/run/user/<user-id>/clipcatd.pid"

# Controls how often the program updates its stored value of the Linux primary selection.
# In the Linux environment, the primary selection automatically updates to reflect the currently highlighted text or object,
# typically updating with every mouse movement.
primary_threshold_ms = 5000

[log]
# Emit log messages to a log file.
# If this value is omitted, `clipcatd` will disable logging to a file.
file_path = "/path/to/log/file"

# Emit log messages to systemd-journald.
emit_journald = true

# Emit log messages to stdout.
emit_stdout = false

# Emit log messages to stderr.
emit_stderr = false

# Log level.
level = "INFO"

[watcher]
# Enable watching the X11 clipboard selection.
enable_clipboard = true

# Enable watching the X11 primary selection.
enable_primary = true

# Ignore clips that match any of the X11 `TARGETS`.
sensitive_x11_atoms = ["x-kde-passwordManagerHint"]

# Ignore text clips that match any of the provided regular expressions.
# The regular expression engine is powered by https://github.com/rust-lang/regex.
denied_text_regex_patterns = []

# Ignore text clips with a length less than or equal to `filter_text_min_length`, in characters (Unicode scalar value), not bytes.
filter_text_min_length = 1

# Ignore text clips with a length greater than `filter_text_max_length`, in characters (Unicode scalar value), not bytes.
filter_text_max_length = 20000000

# Enable or disable capturing images.
capture_image = true

# Ignore image clips with a size greater than `filter_image_max_size`, in bytes.
filter_image_max_size = 5242880

[grpc]
# Enable gRPC over HTTP.
enable_http = true

# Enable gRPC over Unix domain socket.
enable_local_socket = true

# Host address for gRPC.
host = "127.0.0.1"

# Port number for gRPC.
port = 45045

# Path for the Unix domain socket.
# If this value is omitted, `clipcatd` will place the socket in `$XDG_RUNTIME_DIR/clipcat/grpc.sock`.
local_socket = "/run/user/<user-id>/clipcat/grpc.sock"

[dbus]
# Enable D-Bus.
enable = true

# Specify the identifier for the current `clipcat` instance.
# The D-Bus service name will appear as "org.clipcat.clipcat.instance-0".
# If the identifier is not provided, the D-Bus service name will appear as "org.clipcat.clipcat".
identifier = "instance-0"

[desktop_notification]
# Enable desktop notifications.
enable = true

# Path for an icon; the given icon will be displayed in the desktop notification,
# if your desktop notification server supports showing an icon.
# If this value is not provided, the default value `accessories-clipboard` will be used.
icon = "/path/to/the/icon"

# Timeout duration in milliseconds.
# This sets the time from when the notification is displayed until it is closed by the notification server.
timeout_ms = 2000

# Define the length of long plaintext.
# If the length of plaintext is greater than or equal to `long_plaintext_length`,
# a desktop notification will be emitted.
# If this value is 0, no desktop notification will be emitted for long plaintext.
long_plaintext_length = 2000


# Snippets, only UTF-8 text is supported.
[[snippets]]
[snippets.Directory]
# Name of snippet.
name = "my-snippets"
# File path to the directory containing snippets.
path = "/home/user/snippets"

[[snippets]]
[snippets.File]
# Name of snippet.
name = "os-release"
# File path to the snippet.
path = "/etc/os-release"

[[snippets]]
[snippets.Text]
# Name of snippet.
name = "cxx-io-speed-up"
# Content of the snippet.
content = '''
int io_speed_up = [] {
    std::ios::sync_with_stdio(false);
    std::cin.tie(nullptr);
    std::cout.tie(nullptr);
    return 0;
}();
'''

[[snippets]]
[snippets.Text]
name = "rust-sieve-primes"
content = '''
fn sieve_primes(n: usize) -> Vec<usize> {
    if n < 2 {
        return Vec::new();
    }
    let root_n = f64::from(n as i32).sqrt().floor() as usize;
    let mut is_prime = vec![true; n + 1];
    for i in 2..=root_n {
        if !is_prime[i] {
            continue;
        }
        for j in ((i << 1)..=n).step_by(i) {
            is_prime[j] = false;
        }
    }
    is_prime
        .into_iter()
        .enumerate()
        .skip(2)
        .filter_map(|(i, x)| if x { Some(i) } else { None })
        .collect()
}
'''
```

</details>

<details>
    <summary>Configuration for <b>clipcatctl</b></summary>

```toml
# Server endpoint.
# `clipcatctl` connects to the server via a Unix domain socket if `server_endpoint` is a file path, such as:
# "/run/user/<user-id>/clipcat/grpc.sock".
# It connects via HTTP if `server_endpoint` is a URL, like: "http://127.0.0.1:45045".
server_endpoint = "/run/user/<user-id>/clipcat/grpc.sock"

[log]
# Emit log messages to a log file.
# Delete this line to disable logging to a file.
file_path = "/path/to/log/file"
# Emit log messages to systemd-journald.
emit_journald = true
# Emit log messages to stdout.
emit_stdout = false
# Emit log messages to stderr.
emit_stderr = false
# Log level.
level = "INFO"
```

</details>

<details>
    <summary>Configuration for <b>clipcat-menu</b></summary>

```toml
# Server endpoint
# The `clipcat-menu` connects to the server via a Unix domain socket if `server_endpoint` is a file path, such as:
# "/run/user/<user-id>/clipcat/grpc.sock".
# It connects via HTTP if `server_endpoint` is a URL, like: "http://127.0.0.1:45045".
server_endpoint = "/run/user/<user-id>/clipcat/grpc.sock"

# The default finder to invoke when no "--finder=<finder>" option is provided.
finder = "rofi"

[log]
# Emit log messages to a log file.
# Delete this line to disable logging to a file.
file_path = "/path/to/log/file"
# Emit log messages to systemd-journald.
emit_journald = true
# Emit log messages to stdout.
emit_stdout = false
# Emit log messages to stderr.
emit_stderr = false
# Log level.
level = "INFO"

# Options for "rofi".
[rofi]
# Length of line.
line_length = 100
# Length of menu.
menu_length = 30
# Prompt for the menu.
menu_prompt = "Clipcat"
# Extra arguments to pass to `rofi`.
extra_arguments = ["-mesg", "Please select a clip"]

# Options for "dmenu".
[dmenu]
# Length of line.
line_length = 100
# Length of menu.
menu_length = 30
# Prompt for the menu.
menu_prompt = "Clipcat"
# Extra arguments to pass to `dmenu`.
extra_arguments = [
  "-fn",
  "SauceCodePro Nerd Font Mono-12",
  "-nb",
  "#282828",
  "-nf",
  "#ebdbb2",
  "-sb",
  "#d3869b",
  "-sf",
  "#282828",
]

# Customize your finder.
[custom_finder]
# External program name.
program = "fzf"
# Arguments for calling the external program.
args = []

```

</details>

## Integration

<details>
    <summary>Integrating with <a href="https://www.zsh.org/" target="_blank">Zsh</a></summary>

For `zsh` users, it is useful to integrate `clipcat` with `zsh`.

Add the following commands to your `zsh` configuration file (`~/.zshrc`):

```bash
if type clipcat-menu >/dev/null 2>&1; then
    alias clipedit=' clipcat-menu --finder=builtin edit'
    alias clipdel=' clipcat-menu --finder=builtin remove'

    bindkey -s '^\' "^Q clipcat-menu --finder=builtin insert ^J"
    bindkey -s '^]' "^Q clipcat-menu --finder=builtin remove ^J"
fi
```

</details>

<details>
    <summary>Integrating with <a href="https://i3wm.org/" target="_blank">i3 Window Manager</a></summary>

For `i3` window manager users, it is useful to integrate `clipcat` with `i3`.

Add the following options to your `i3` configuration file (`$XDG_CONFIG_HOME/i3/config`):

```

exec_always --no-startup-id clipcatd # start clipcatd at startup

set $launcher-clipboard-insert clipcat-menu insert
set $launcher-clipboard-remove clipcat-menu remove

bindsym $mod+p exec $launcher-clipboard-insert
bindsym $mod+o exec $launcher-clipboard-remove

```

**NOTE**: You can use `rofi` or `dmenu` as the default finder.

</details>

<details>
    <summary>Integrating with <a href="http://leftwm.org/" target="_blank">LeftWM</a></summary>

For `leftwm` users, it is useful to integrate `clipcat` with `leftwm`.

Add the following keybindings to your `leftwm` configuration file (`$XDG_CONFIG_HOME/leftwm/config.ron`):

```ron
(
    /* other configurations */
    keybind: [
        /* select clip from clipboard */
        (command: Execute, value: "clipcat-menu insert", modifier: ["modkey"], key: "p"),
        (command: Execute, value: "clipcat-menu remove", modifier: ["modkey"], key: "o"),
        /* other configurations */
    ],
    /* other configurations */
)
```

**NOTE**: You can use `rofi` or `dmenu` as the default finder.

Add the following commands to your `$XDG_CONFIG_HOME/leftwm/themes/current/up`:

```bash
# other configurations

# Start clipcatd
clipcatd

# other configurations
```

Add the following commands to your `$XDG_CONFIG_HOME/leftwm/themes/current/down`:

```bash
# other configurations

# Terminate clipcatd
pkill clipcatd

# other configurations
```

</details>

<details>
    <summary>Starting <b>clipcatd</b> with <a href="https://systemd.io/" target="_blank">systemd</a></summary>

Put the following snippet in `$XDG_CONFIG_HOME/systemd/user/clipcat.service`:

```
[Unit]
Description=Clipcat Daemon
PartOf=graphical-session.target

[Install]
WantedBy=graphical-session.target

[Service]
# NOTE: We assume that your `clipcatd` is located at `/usr/bin/clipcatd`.
ExecStartPre=/bin/rm -f %t/clipcat/grpc.sock
ExecStart=/usr/bin/clipcatd --no-daemon --replace
Restart=on-failure
Type=simple
```

Enable and start `clipcat` with the following commands:

```bash
systemctl --user daemon-reload
systemctl --user enable clipcat.service
systemctl --user start clipcat.service
systemctl --user status clipcat.service
```

</details>

## Programs in this Repository

- `clipcatd`: The `clipcat` server (daemon).
- `clipcatctl`: The `clipcat` client that provides a command line interface.
- `clipcat-menu`: The `clipcat` client that utilizes a built-in or external finder to select clips.

- `clipcat-notify`: A tool for monitoring clipboard events. It watches the clipboard and exits when a change is detected, returning an exit code of 0 for success and 1 for errors.

> [!Note]
> clipcat-notify does not interact with `clipcatd`, `clipcatctl`, or `clipcat-menu`; it is simply a tool for monitoring the clipboard.

## License

Clipcat is licensed under the GNU General Public License version 3. See [LICENSE](./LICENSE) for more information.
