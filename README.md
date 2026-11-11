# Vellmark

An open-world RPG where you rewrite reality with a word. This repository is building the Showcase first.

Start with [START_HERE.md](START_HERE.md).

## Running it on a Mac

**Without installing anything:** on GitHub, open **Actions**, the newest green **CI** run, and download `vellmark-macos-N` under **Artifacts**. Unzip it, right-click `Vellmark.app`, and choose **Open** twice.

**From the source:** you need Apple's command line tools, CMake, and Rust. SDL3 is built from source, which is why CMake is needed; a build that stops with "is `cmake` not installed?" is missing it.

```bash
xcode-select --install
brew install cmake
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Keep the folder's path free of spaces: CMake builds can fail in a folder such as `vellmark-main 2`. Then, inside the folder:

```bash
cargo run --release -p vellmark-app
```

The first build takes several minutes.

| Key | What it does |
|---|---|
| W A S D, mouse | Move and look |
| R (hold) | Close reading |
| F6 | Step through the depth looks: living (the default), flat, atoms, lattice, source |
| \` then `heat door 0.9` | Heat a thing; up close its letters say so (`-0.8` freezes it) |
| F7 | Break the nearest breakable thing (the door) into its letters; `break door 0.1` in the console crumbles it into a heap |
| F5 | Light the lantern or put it out |
| F4 | The tuning panel |
| F1 | The profiler |
| ` | The console. Try `hour 23` |
| Escape | Free the mouse; again to leave |

