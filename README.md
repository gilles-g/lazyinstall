# lazyinstall

A terminal UI — in the spirit of _lazygit_ — to **keep your hand-installed tools up to date**.

You already have folders holding an update script (`update-lazygit.sh`, `update-composer.sh`, …).
`lazyinstall` tracks those folders, runs their script on demand, and streams the output live,
so updating five tools is five keystrokes instead of five `cd` + `./update-*.sh`.

Scripts run inside a real **pseudo-terminal**, so a `sudo` inside your script behaves: the password
prompt is detected, asked for once in a masked popup, then reused for the rest of the session.

## Preview

```
┌ lazyinstall — dossiers suivis ───────────────────────────────────────────┐
│ ● lazygit           à jour    /home/user/lazygitinstall                  │
│ ○ composer          au repos  /home/user/lazycomposerinstall             │
│ ✗ neovim            ÉCHEC     /home/user/nvim-install                    │
└──────────────────────────────────────────────────────────────────────────┘
┌ Sortie : lazygit ────────────────────────────────────────────────────────┐
│  Version installée : 0.62.1                                              │
│  Dernière release  : 0.62.2                                              │
│  lazygit mis à jour : 0.62.1 -> 0.62.2                                   │
└──────────────────────────────────────────────────────────────────────────┘
 [Enter/u] màj  [U] tout  [a] ajouter  [d] retirer  [j/k] naviguer  [q] quitter  [?] aide
```

The TUI itself speaks French. Status markers: `○` idle · `◌` updating · `●` up to date · `✗` failed.

## Install in 2 steps

### 1. Get Rust (skip if `cargo --version` already answers)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install `lazyinstall`

```bash
cargo install --git https://github.com/gilles-g/lazyinstall.git --locked
```

That's it — the binary lands in `~/.cargo/bin/lazyinstall`. Run it with:

```bash
lazyinstall
```

> If the command isn't found, `~/.cargo/bin` is missing from your `PATH`:
> `export PATH="$HOME/.cargo/bin:$PATH"` in your `~/.zshrc` / `~/.bashrc`.

To update lazyinstall itself later, re-run step 2 — `cargo install` overwrites the binary.

<details>
<summary>From a local clone (for hacking on it)</summary>

```bash
git clone https://github.com/gilles-g/lazyinstall.git
cd lazyinstall
cargo install --path . --locked   # install
cargo run                         # or just run it from source
```

</details>

## Quick start

Point lazyinstall at a folder that holds an update script:

```
~/lazygitinstall/
├── update-lazygit.sh   ← the script lazyinstall will run
└── current/            ← whatever your script installs
```

Then, inside the TUI:

1. press `a`, type the folder path (`~` is expanded), `Enter`
2. press `Enter` (or `u`) to run its update — output streams in the bottom panel

The folder is remembered, so next time it's just step 2.

### How a folder becomes an entry

A tracked folder must contain at least one `*.sh` script:

- every `update-*.sh` becomes its own entry — one folder can hold several tools;
- if there's no `update-*.sh`, the first `*.sh` found is used;
- the displayed name comes from the script (`update-lazygit.sh` → `lazygit`), falling back to the
  folder name.

Scripts are run with `bash <script>`, from their own folder as working directory, with `LC_ALL=C`
(so the sudo prompt stays detectable). No execute bit needed.

## Keys

| Key             | Action                                          |
|-----------------|-------------------------------------------------|
| `j` / `k`, ↓/↑  | move through the list                           |
| `Enter` / `u`   | update the selected folder                      |
| `U`             | update everything (all in parallel)             |
| `a`             | add a folder to track (type its path)           |
| `d`             | stop tracking the selected folder               |
| `?`             | toggle help                                     |
| `Esc`           | close help / cancel the add or password prompt  |
| `q` / `Ctrl-C`  | quit                                            |

### When a script needs sudo

If the running script asks for a password, a masked popup opens, labelled with the target it belongs
to. Type it, `Enter`. The password is kept **in memory for the session only** — reused for the other
targets, never written to disk, and wiped when you quit. `Esc` cancels the prompt and kills the
running script.

## Configuration

The list of tracked folders lives in:

```
~/.config/lazyinstall/targets.json
```

Nothing else is persisted. Folders whose script has vanished are dropped from the file at startup,
and a toast tells you how many were removed.

## Development

```bash
cargo test     # discovery + full launch → streaming → state flow, no network, no TUI
cargo clippy
cargo fmt
```

`CLAUDE.md` documents the domain model (`src/install/`) and the UI split (`src/ui/`).

## License

MIT.
