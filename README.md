# tamadoro

vibe coded trash software because all the pomodoros ive used before felt bad

lowk this is actually somehow working for me.

A terminal Pomodoro timer with a Tamagotchi-style pet that levels up while you focus, gets hungry if you ignore it, and eventually graduates into a Hall of Fame.

## Features

- 25/5 minute Pomodoro loop with a big ASCII clock
- A pet (Blob / Cat / Robot / Ghost) that evolves through Egg → Baby → Teen → Master as you complete sessions
- Food, hunger, and death — feed it by working, or it starves
- Streak tracking with XP bonuses
- Hall of Fame for graduated pets, with sessions / focus time / dates
- Speech bubbles, mood-based art, native macOS notifications when the pet is hungry or dies
- All state in a single JSON save under your OS data dir

## Install

You need a recent Rust toolchain.

```sh
git clone https://github.com/cosoda/tamadoro
cd tamadoro
cargo build --release
cp target/release/tamadoro ~/.local/bin/   # or anywhere on your PATH
```

Or run it directly without installing:

```sh
cargo run --release
```

## Usage

```
tamadoro            # normal mode
tamadoro --test     # unlocks the Debug tab and a separate save.test.json
```

### Keys

- `Space` — start / pause the current session
- `r` — reset the timer (Timer tab only)
- `Tab` / `Shift-Tab`, `h`/`l`, `←`/`→` — switch tabs
- `j`/`k`, `↑`/`↓` — browse the Hall of Fame
- `q` — quit

### Debug keys (only with `--test`)

`1`/`2` add XP, `3` levels up, `4` jumps evolution stage, `5` bumps streak, `6` cycles pet type, `7`/`8` add/remove food, `9` toggles death, `s` force-completes a session, `n` spawns a new pet, `0` wipes all data.

## Save location

`~/Library/Application Support/tamadoro/save.json` on macOS (uses `dirs::data_local_dir()` elsewhere). `--test` mode clones it to `save.test.json` so experimenting doesn't touch your real pet.
