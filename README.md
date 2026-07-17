# Wayland Web Idle

Tiny web server to know for how long the current Wayland session has been idle.

I created this tool use with Home Assistant, so that it would automatically
turn on/off the lights in my office automatically when I'm at my computer.

## Usage

```
./whidle
```

By default, whidle runs on port 2323, but this can be changed by
setting the `PORT` environment variable. It keeps running in the
foreground and will log requests that it receives.

Test it with a simple curl request:

```
curl localhost:2323
{"idleseconds": 17}
```

Requests are logged to stdout, with the `idleseconds` value from the
reply appended at the end:

```
127.0.0.1 "GET / HTTP/1.1" 200 18 idleseconds=17
```

## Caveat

This does not report the actual idle time, but the idle time *since the
program was started*. This is because Wayland doesn't expose the
necessary information; so all we can do is run a program that asks
Wayland "tell us whenever the user is idle/busy" and maintain the relevant
timer on our side.

## Building

Written in Rust, using the `ext-idle-notify-v1` Wayland protocol (via
[`wayrs-client`](https://github.com/MaxVerevkin/wayrs)) to track idle time.

To build this code, install a standard rust build toolchain (package `rust`
on most distros), then:

```
cargo build --release
```

This produces the binary at `target/release/whidle`.

## AI disclosure

The code was generated in a single session with Claude Code. This is what `/usage` says:

```
  Total cost:            $9.47
  Total duration (API):  21m 41s
  Total duration (wall): 8h 34m 27s
  Total code changes:    311 lines added, 82 lines removed
  Usage by model:
      claude-haiku-4-5:  529 input, 13 output, 0 cache read, 0 cache write ($0.0006)
       claude-sonnet-5:  703 input, 104.7k output, 19.9m cache read, 324.8k cache write ($9.47)
```
