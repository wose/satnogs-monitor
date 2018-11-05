# SatNOGS Monitor (WIP)

![Screenshot](/doc/screen.png)

## WIP

This is more of a proof of concept for now. There is virtually no configuration
and the polled stations are hard coded. I'm playing with different concepts and
layouts, so expect everything to change rapidly. So it's not really useable yet,
but feel free to open issues or find me on the [satnogs irc
channel](https://satnogs.org/contact/) if you have suggestions what info about
your station is useful to you and should be included.

## TODOs / planned features

Note: the list is by no means complete or in any particular order.

- [ ] visual alerts on station failure (failed obs, no heartbeats, ...)
- [ ] rotator state
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

Use your distribution package management to install ```rustup``` if possible.
See [Install Rust](https://www.rust-lang.org/en-US/install.html).

### A true color terminal

While other terminals will be supported in the future, the screenshot was taken
using [alacritty](https://github.com/jwilm/alacritty) with the [Lucy
Tewi](https://github.com/lucy/tewi-font) font.

## Hacking

```
git clone https://github.com/wose/satnogs-monitor.git
cd satnogs-monitor/monitor
cargo run --release
```

