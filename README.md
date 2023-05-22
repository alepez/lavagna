# lavagna

> It's a blackboard, not a lasagna.

![preview](.lavagna.gif)

*Lavagna* is a "no frills", collaborative, blackboard, ideal for simple
sketches during online meetings. You have just a black screen, without icons or
buttons cluttering your beautiful drawings.

Colors and line width can be controlled by the keyboard, or by enabling a simple
toolbar if the keyboard is not available.

Just you, your peers, and your chalk.

## Keyboard bindings

| Button | Action   | Note                                 |
|--------|----------|--------------------------------------|
| C      | Color    | Change the chalk color               |
| M      | Grow     | Grow chalk size 2x                   |
| N      | Shrink   | Shrink chalk size 2x                 |

## Installation

[Archives of precompiled binaries for *lavagna* are available for Windows, macOS
and Linux.](https://github.com/alepez/lavagna/releases/latest)

### Install from source

If you're a **Rust programmer**, *lavagna* can be installed with `cargo`.

- Note that the minimum supported version of Rust for *lavagna* is 1.61.0,
  although *lavagna* may work with older versions.

To install from sources:

```shell
git clone https://github.com/alepez/lavagna.git
cd lavagna
cargo install --path lavagna --locked
```

### Install from crates.io

To install from [crates.io](https://crates.io):

```shell
cargo install lavagna
```

Note that the version available in *crates.io* may be older than the one you
find on [latest release
page](https://github.com/alepez/lavagna/releases/latest).

## Instant collaboration

*lavagna* uses *WebRtc* for instant collaboration. So you can use it for online
meetings. You don't even need to install anything, because *lavagna* is available
as a web-app too.

You need a signaling server
like [`matchbox_server`](https://github.com/johanhelsing/matchbox/tree/main/matchbox_server)
installed somewhere.

Don't worry, for quick testing you can borrow *devand.dev* signaling server:

```shell
lavagna --collab-url wss://lavagna.devand.dev/YOUR_ROOM
```

Change `YOUR_ROOM` to whatever you prefer. If you do the same from a different
computer, whatever you draw is visible on the other side (and vice versa).

## Desktop app development

`lavagna` works on many operating systems:

- Linux (x86)
- Linux (ARM)
- macOS (x86)
- Windows

You just need to have a Rust toolchain installed. Run this from the project
directory:

```shell
cargo run
```

To print help information, just use:

```shell
cargo run -- --help
```

