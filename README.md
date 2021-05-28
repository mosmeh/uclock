# uclock

[![build](https://github.com/mosmeh/uclock/workflows/build/badge.svg)](https://github.com/mosmeh/uclock/actions)

Unix time clock in terminal

![](screenshot.png)

## curl version

```bash
cargo run -p uclock-server
```

```bash
curl localhost:8080
```

## Standalone version

```bash
cargo run -p uclock-cli
```

Inspired by [tty-clock](https://github.com/xorg62/tty-clock)
