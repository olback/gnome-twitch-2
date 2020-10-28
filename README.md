# [WIP] Gnome Twitch 2

Inspired by [vinszent/gnome-twitch](https://github.com/vinszent/gnome-twitch).


## Why?

The original is no longer maintained and it uses the deprecated Twitch API.


## Building

Tools required to build:
* `cargo`, `rustc` (easiest to install with [`rustup`]("https://rustup.rs/"))
* [`glib-compile-resources`]("https://developer.gnome.org/gio//2.34/glib-compile-resources.html")

```
cargo [run|build] [--release]
```

## Installing

Tools required to install:
* [`Meson`]("https://mesonbuild.com/")
* [`ninja`]("https://ninja-build.org/")
* [`glib-compile-schemas`]("https://developer.gnome.org/gio//2.34/glib-compile-schemas.html")
* [`gtk-update-icon-cache`]("https://developer.gnome.org/gtk3/stable/gtk-update-icon-cache.html")

```
cargo install --path .
meson build
sudo ninja -C build install
```
