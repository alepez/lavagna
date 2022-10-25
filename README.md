# lavagna

> It's a blackboard, not a lasagna.

![preview](.lavagna.gif)

*Lavagna* is a "no frills" blackboard, ideal for simple sketches during online
meetings. You have just a black screen, without icons or buttons cluttering
your beautiful drawings. 

Just you and your chalk.

## Keyboard shortcuts

| Button | Action   | Note                                 |
|--------|----------|--------------------------------------|
| Esc    | Quit     | Quit the application                 |
| X      | Clear    | Take a snapshot and clear everything |
| C      | Color    | Change the chalk color               |
| U      | Undo     | Resume the last snapshot             |
| S      | Snapshot | Take a snapshot                      |
| M      | Grow     | Grow pen size 2x                     |
| N      | Shrink   | Shrink pen size 2x                   |

## Instant collaboration

*lavagna* can use WebRtc for instant collaboration. Try it:

```shell
cargo run --bin lavagna -- --collab-url wss://lavagna-server.herokuapp.com/YOUR_ROOM
```

Change `YOUR_ROOM` to whatever you prefer.

You can setup your own server by using
[lavagna_server](https://github.com/alepez/lavagna_server) crate.
