# SatNOGS Monitor (WIP)

![Screenshot](/doc/screen.png)

## WIP

This is more of a proof of concept for now. I'm playing with different
architectures and layouts, so expect everything to change rapidly. Feel free to
open issues or find me on the [SatNOGS irc
channel](https://satnogs.org/contact/) if you have suggestions what info about
your station is useful to you and should be included. There is also a
corresponding forum post at the [SatNOGS community
forum](https://community.libre.space/t/satnogs-station-monitor/2802)

## TODOs / planned features

Note: the list is by no means complete or in any particular order.

- [X] reduce API queries
- [X] calculate ground tracks only when a new orbit begins
- [X] show satellite footprint (can be toggled)
- [X] polar plot
- [X] refactor station info, obs info, etc. into separate widgets
- [ ] detect supported colors and change palette accordingly
- [X] build debian package for the RPi SatNOGS image
  - Check [releases](https://github.com/wose/satnogs-monitor/releases)
- [ ] visual alerts on station failure (failed obs, no heartbeats, ...)
- [ ] rotator state
- [X] support multiple stations
- [ ] theme support
- [ ] network overview
- [ ] GUI
- [ ] cross platform
- [ ] waterfall stream of current observation
- [ ] audio stream of current observation

## Dependencies

### libgpredict

See [libgpredict](https://github.com/cubehub/libgpredict) for details.

```
git clone https://github.com/cubehub/libgpredict.git
cd libgpredict
mkdir build
cd build
cmake ../
make
make install
sudo ldconfig # for linux
```

### Rust

Use your distribution package management to install `rust` or `rustup` if
possible. See [Install Rust](https://www.rust-lang.org/en-US/install.html).

```
rustup install stable
```

If you want to build the monitor on the rpi you'll have to settle with version
`1.37.0` for now:

```
rustup install 1.37.0
```

For more information see rust-lang/rust#62896

### A true color terminal

While other terminals will be supported in the future, the screenshot was taken
using [alacritty](https://github.com/jwilm/alacritty) with the [Lucy
Tewi](https://github.com/lucy/tewi-font) font.

Check [the wiki](https://github.com/wose/satnogs-monitor/wiki) for infos on
other terminals with and without Xorg.

## Building

```
git clone https://github.com/wose/satnogs-monitor.git
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

