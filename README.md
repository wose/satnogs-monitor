# SatNOGS Monitor (WIP)

![Screenshot](doc/screen.png)

[![builds.sr.ht status](https://builds.sr.ht/~wose/satnogs-monitor.svg)](https://builds.sr.ht/~wose/satnogs-monitor?)

## WIP

This is more of a proof of concept for now. I'm playing with different
architectures and layouts, so expect everything to change rapidly. Feel free to
open issues or find me on the [SatNOGS irc
channel](https://satnogs.org/contact/) if you have suggestions what info about
your station is useful to you and should be included. There is also a
corresponding forum post at the [SatNOGS community
forum](https://community.libre.space/t/satnogs-station-monitor/2802)

## Dependencies

### Rust

Use your distribution package management to install `rust` or `rustup` if
possible. See [Install Rust](https://www.rust-lang.org/en-US/install.html).

```
rustup install stable
```

### A terminal

While other terminals will be supported in the future, the screenshot was taken
using [alacritty](https://github.com/jwilm/alacritty) with the [Lucy
Tewi](https://github.com/lucy/tewi-font) font. Any
[nerd-fonts](https://github.com/ryanoasis/nerd-fonts) font should work as well.
You'll need a terminal emulator which supports trur colors for the waterfall
widget to look nice.


Check [the wiki](https://github.com/wose/satnogs-monitor/wiki) for infos on
other terminals with and without Xorg.

## Building

```
git clone --recursive https://github.com/wose/satnogs-monitor.git
cd satnogs-monitor/monitor
mkdir ~/.config/satnogs-monitor
cp examples/config.toml ~/.config/satnogs-monitor/
edit ~/.config/satnogs-monitor/config.toml
cargo run --release
```

The config file is optional, you can also provide stations with the `-s`
parameter. At least one station has to be provided by either a config file or
the command line.

```
cargo run --release -- -s 175 -s 227
```

## Keys

Key            | Description
---------------|------------
`f` | toggle satellite footprint
`l` | toggle log window
`\t` | next station
`q`, `ctrl-c` | quit

## Docker

Building the docker container
```
docker build -t satnogs-monitor:latest .
```

Running
```
docker run -it --rm -e STATION_ID=1492 satnogs-monitor:latest
```
