# Scenable

Scenable is a custom scenery manager for the [X-Plane 11](https://www.x-plane.com/) flight simulator.

## Why Scenable?

Currently X-Plane appears to waste a lot of memory loading scenery and scenery library assets which it doesn't actually use. I measured a difference of about 10GB in memory usage when flying in an area with no custom scenery, but with lots of custom scenery in other areas currently enabled. Having lots of custom scenery enabled also makes the simulator slower to start.

The ultimate (**not yet complete**) goal of this tool is to make the management of enabling/disabling custom scenery based on the area that you want to fly in as painless as possible. When disabling scenery with this tool, it should detect whether scenery libraries are currently un-used and will disable them automatically, and the same thing in reverse, when enabling a scenery pack that uses a disabled library, it will also enable the library as required.

## Installation

You can build `scenable` application with the [rust compiler](https://www.rust-lang.org/) and run it with the following command:

```
cargo run --release
```

It's also possible to install the binary to your system using:

```
cargo install --git https://github.com/kellpossible/scenable
```

And then run using the `scenable` command.

## Roadmap

Core features:

- [x] ~Create GUI to manually enable/disable scenery packs.~
- [ ] Scenery pack selection (including multiple selection).
- [ ] Capability to re-order scenery packs (re-order selected scenery packs using buttons on the top toolbar).
- [ ] Preset creation, selection, updating and saving.
- [ ] Indicate differences between current state and saved state.
- [ ] Detection of new scenery packs, and removed scenery packs.
- [ ] Parse scenery library txt files.
- [ ] Implement a dsf parser (perhaps wrap <https://github.com/X-Plane/xptools/tree/master/src/DSF>, or just implement the required subset of the [DSF specification](https://developer.x-plane.com/article/dsf-file-format-specification/) in pure Rust).
- [ ] Class sceneries as libraries, textures, airports, etc (apply categories and show in UI)
- [ ] Scan DSF files (including unzipping using `rust-lzma`) to find references to scenery library objects.
- [ ] Automatic enable/disable of library scenery packs based on calculated dependencies.

Nice to have:

- [ ] Show coverage map.
- [ ] Parse x-plane navigation database.
- [ ] Enter route, automatically only enable scenery along route.
