# lavagna

> It's a blackboard, not a lasagna.

![preview](.lavagna.gif)

*Lavagna* is a collaborative blackboard, perfect for creating simple sketches
during online meetings. It has an (optional) minimal toolbar which can be
hidden to avoid distractions from your drawings. You can control colors and
line width using the keyboard or the toolbar.

It's just you, your peers, and your creativity.

## Keyboard bindings

| Button | Action  | Note                       |
|--------|---------|----------------------------|
| C      | Color   | Change the chalk color     |
| M      | Grow    | Grow chalk size 2x         |
| N      | Shrink  | Shrink chalk size 2x       |
| X      | Clear   | Clear the whole blackboard |
| U      | Toolbar | Toggle toolbar visibility  |

## Installation

[Archives of precompiled binaries for *lavagna* are available for Windows, macOS
and Linux.](https://github.com/alepez/lavagna/releases/latest)

### Install from source

If you're a **Rust programmer**, *lavagna* can be installed with `cargo`.

- Note that the minimum supported version of Rust for *lavagna* is 1.76,
  although *lavagna* may work with older versions.

To install from sources:

```shell
git clone https://github.com/alepez/lavagna.git
cd lavagna
cargo install --path lavagna --locked
```

### Install from crates.io

Currently, installing from crates.io is not
available. [See this issue.](https://github.com/alepez/lavagna/issues/21)

## Instant collaboration

*lavagna* uses *WebRtc* for instant collaboration. So you can use it for online
meetings. You don't even need to install anything, because *lavagna* is
available as a web-app too.

You need a signaling server
like [`matchbox_server`](https://github.com/johanhelsing/matchbox/tree/main/matchbox_server)
installed somewhere.
See [this post by Johan Helsing](https://johanhelsing.studio/posts/deploying-matchbox/)

Don't worry, for quick testing you can borrow my signaling server:

```shell
lavagna --collab-url ws://lavagna.alepez.dev:3536/demo
```

Change `demo` to your preferred name or a unique id. If you do the same on
different device, anything you draw will be visible on the other side (and vice
versa).

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

## Web app development

This script builds all the assets needed for a web application. You'll find them
on `www` directory, so you can use any http server able to serve static files.

```shell
./tools/build-web
cd www
python -m http.server 8000
```

See also `docker/lavagna-webapp/Dockerfile` as an example of how to publish the
web-app.
