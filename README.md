# Vellmark

An open-world RPG where you rewrite reality with a word. This repository is building the Showcase first.

Start with [START_HERE.md](START_HERE.md).

## Running it

**Ready-made builds, nothing to install:** on GitHub, open **Releases** and take the newest
**Showcase build**. It is a draft that only people with push access can see, made from the newest green `main`,
with a build for each system:

- **macOS:** unzip `vellmark-macos-*.zip`, right-click `Vellmark.app`, choose **Open** twice. `valley.command` starts in the valley.
- **Windows:** extract `vellmark-windows-*.zip`, double-click `Vellmark.exe`, or `Valley.bat` for the valley.
- **Linux:** unpack `vellmark-linux-*.tar.gz`, run `Vellmark/vellmark`, or `Vellmark/valley.sh` for the valley.

**From the source, in one double-click:** `run-mac.command` (right-click, **Open** the first
time), `run-windows.bat`, or `run-linux.sh`. Each checks what the build needs and says how to
install what is missing, takes the newest code if this is a git checkout, builds (several
minutes the first time), and starts in the valley. A folder whose path has a space in it is
handled: the build goes elsewhere. What they need:

- **macOS:** Apple's command line tools (`xcode-select --install`), CMake (`brew install cmake`), and Rust (https://rustup.rs).
- **Windows:** Rust from https://rustup.rs with its default Visual Studio build tools, and CMake (`winget install Kitware.CMake`).
- **Linux:** Rust, CMake, and the libraries in the `apt-get` line the script prints.

By hand, from the folder (keep its path free of spaces):

```bash
cargo run --release -p vellmark-app --features dev -- --scene valley
```

Leave out `-- --scene valley` to start in the yard.

`--features dev` brings the development tools: the F keys and the console below. Without it the build is the one a player would get.

The first build takes several minutes.

| Key | What it does |
|---|---|
| W A S D, mouse | Move and look |
| R (hold) | Close reading |
| F6 (fn + F6 on a Mac) | Step through the depth looks: living (the default), flat, atoms, lattice, source. Seen within arm's reach; hold R |
| \` then `heat door 0.9` | Heat a thing; up close its letters say so (`-0.8` freezes it) |
| \` then `rain 1` | Rain, from `0` (dry) to `1` (a downpour); the ground soaks over a minute and dries over eight |
| \` then `preset low` | Choose a preset: `low`, `medium` (the default), or `high`. `--preset high` at launch does the same |
| \` then `wind 0` | A gust writes one of the wind's sentences (`0` to `5`) across the grass ahead of you |
| `--scene valley` (on the command line) | Start on Tithe Hill in the valley instead of the yard |
| F7 | Break the nearest breakable thing (the door) into its letters; `break door 0.1` in the console crumbles it into a heap |
| F5 | Light the lantern or put it out |
| F4 | The tuning panel |
| F1 | The profiler |
| ` | The console. Try `hour 23` |
| Escape | Free the mouse; again to leave |

