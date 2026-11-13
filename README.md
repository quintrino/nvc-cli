# nvc-cli

Look up [Nonviolent Communication](https://www.cnvc.org) feelings and needs from the terminal, including "faux feelings": words like *betrayed*, *offended* or *misunderstood* that sound like feelings but are really interpretations of what someone did. For each one, nvc-cli lists the real feelings and needs that usually sit behind it.

```console
$ nvc-cli offended
😤 Offended  (Not a feeling)
   Interpretation that someone insulted or crossed you.

Likely feelings
  🤕 Hurt — Wounded emotionally.
  😠 Angry — Umbrella word covering enraged, furious, incensed, indignant and more.
  😳 Embarrassed — Umbrella word covering ashamed, chagrined, flustered, guilty and more.

Likely needs
  🎩 Respect — To be treated as worthy.
  🎁 Consideration — To be thought of by others.
  🫶 Acceptance — To be taken as you are.
  🎖️ Dignity — To be treated as having inherent worth.
```

Everything runs offline; the data is built into the binary.

## Install

With Homebrew:

```sh
brew install quintrino/tap/nvc-cli
```

Or build it yourself. You need [Rust](https://rustup.rs) 1.70 or newer.

```sh
cargo install --path .        # installs nvc-cli into ~/.cargo/bin
```

## Usage

```
nvc-cli [OPTIONS] <WORD>...
nvc-cli [OPTIONS] feelings [FILTER]
nvc-cli [OPTIONS] needs [FILTER]
```

| Command | Result |
| --- | --- |
| `nvc-cli betrayed` | The word, then the likely feelings and needs behind it. A real feeling shows similar feelings and likely needs; a need shows the feelings that go with it when it is met or unmet. A leading "I feel…" is ignored. |
| `nvc-cli feelings [filter]` | Every feeling, grouped, optionally filtered by name or group (`angry`, `tired`). |
| `nvc-cli needs [filter]` | Every need, grouped, optionally filtered by name or group (`connection`). |

| Option | Purpose |
| --- | --- |
| `--json` | Print JSON (`kind`, `name`, `emoji`, `description`, `group`, and `role` for lookups). |
| `--no-color` | No bold text. Also off when `NO_COLOR` is set or output is piped. |

Misspelled words get "Did you mean" suggestions. If nothing is close, the exit status is 1.

## Data

All entries live in `data/nvc.json` and are compiled into the binary:

- `feeling_groups`: feelings in groups, each with whether needs are `met` and the `needs` usually involved.
- `need_groups`: needs in groups.
- `faux_feelings`: interpretations, each with the `feelings` and `needs` behind it.

`cargo test` checks that every referenced feeling and need exists. The
Alfred workflow in `alfred/alfred-nvc-workflow` builds from the same file.

## Releases

Every push to `main` runs the tests, builds macOS (Apple silicon and Intel) and Linux binaries, publishes a GitHub release, and rewrites `Formula/nvc-cli.rb` in `quintrino/homebrew-tap` with the new URLs and checksums. That last step needs a `TAP_GITHUB_TOKEN` secret on this repository with write access to the tap.
