# Matrix Rain Terminal Screensaver

A terminal screensaver written in Rust that displays the classic Matrix digital rain effect.

## Features

- Smooth fading trails with glowing heads
- 6 color schemes (green, blue, red, purple, cyan, rainbow)
- Configurable speed, density, and drop length
- Live controls to adjust settings while running
- Handles terminal resize

## Build

```bash
cargo build --release
```

Binary will be at `./target/release/matrix`

## Usage

```bash
./matrix [OPTIONS]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-s, --speed <MS>` | Frame delay in ms (lower = faster) | 50 |
| `-d, --density <0-100>` | Spawn density percentage | 40 |
| `-n, --spawns <N>` | Max spawns per frame | 4 |
| `-l, --length <N>` | Max drop length | 30 |
| `-c, --color <SCHEME>` | Color scheme | green |

### Runtime Controls

| Key | Action |
|-----|--------|
| Up/Down | Adjust speed |
| Left/Right | Adjust density |
| +/- | Adjust drop length |
| 1-6 | Switch color (green/blue/red/purple/cyan/rainbow) |
| q, Esc, Enter, Space, Ctrl+C | Quit |

### Examples

```bash
# Default
./matrix

# Faster and denser
./matrix -s 20 -d 70 -n 8

# Slow and sparse
./matrix -s 80 -d 20 -n 2

# Rainbow mode
./matrix -c rainbow
```

## License

MIT
