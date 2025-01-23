# Minecraft World Finder

Ever wondered where your old Minecraft world back from 20xx went?
This tool will find it for you.

## Setup / Installation

1. First, install Rust according to the instructions given [here](https://www.rust-lang.org/tools/install).
2. `rustup update`
3. `cargo install --git https://github.com/k-gruenberg/minecraft_world_finder`


## Usage

```
$ minecraft_world_finder
```

By default, the tool will search...
* first `%APPDATA%\.minecraft`, then the entire `C:\` on Windows
* first `~/Library/Application Support/minecraft`, then `~`, then the entire `/` on macOS
* first `~/.minecraft`, then `~`, then the entire `/` on Linux

More specifically, this tool searches for any `level.dat` files.

You can also customize the folder(s) to search through:

```
$ minecraft_world_finder <FOLDER1>
$ minecraft_world_finder <FOLDER1> <FOLDER2>
$ minecraft_world_finder <FOLDER1> <FOLDER2> <FOLDER3>
...
```

Folders may overlap, even then, each Minecraft world will only be printed once.
