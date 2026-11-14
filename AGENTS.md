# Defending Earth: rules for AI assistants

A squad-tactics game in Rust on macroquad. The owner is building their
first game and designs it together with the assistant.

## Read first

1. `docs/STATUS.md`: where things stand and what is next.
2. `docs/ROADMAP.md`: graphics phases, the game systems track, and which
   AI model level suits which work.
3. `docs/CODE_MAP.md`: where everything is in the code, and its limits.
4. `docs/LORE.md`: every story and design decision, in the order made.
5. `docs/NAMES.md`: every name in the game. `docs/ART_BIBLE.md`: the look.

## Standing rules

- **Original work only.** Every name, design, text and asset is this
  project's own. Never name, copy or allude to an existing commercial game.
- **Lore is decided with the owner.** Do not invent story or rules. Mark
  anything undecided "to settle" and ask.
- **Keep the engine.** If something cannot be done in macroquad, say so
  before swapping anything.
- **Explain plainly.** Short sentences, no jargon without a one-line
  explanation.
- **Ask decisions as questions with options**, recommended option first,
  not as a list in prose. Write times like 7am and 4:45pm.
- **The sea campaign is deferred.** Keep what exists consistent; do not
  design it further.
- **Who does what.** Design decisions are made by the most capable model
  (Fable). All design and animation jobs (the look, models, pictures,
  motion) are also done by Fable itself, never by a sub-agent. Other
  actions (writing code, editing docs, commits, pushes, batch
  jobs) are handed to a sub-agent on the smallest model that can
  reasonably do them. `docs/ROADMAP.md` lists the level for each kind of
  work.
- **Look for emergent stories.** The heart of this game is players
  finding their own stories on missions. When designing or reviewing any
  system, point out where it could give players such stories, and raise it
  with the owner as a question. Do not add it unasked.
- **Commit or push only when asked.**
- **The repository is public.** Never commit pictures or names of real
  people, soldier-pool data, personal details, keys or passwords.
- **Docs move with the code.** When an on-screen name changes in `src/`,
  change `docs/NAMES.md` in the same edit. Record lore decisions in
  `docs/LORE.md` as they are made.

## Build and test

```bash
cargo build --release
```

The game needs a display, and every launch opens a window on the owner's
screen, so launch it sparingly.

The self-test's argument is the file it WRITES its result to. Point it at
a scratch file, never at a tracked file in `checks/`:

```bash
./target/release/defending_earth --self-test /tmp/selftest_result.json
```

Debug flags (`--clock`, `--weather`, `--site`, `--screenshot` and others)
are listed in `docs/CODE_MAP.md`.

## Things that bite

- On-screen words and code names differ: Spine is `lattice`, Glow is
  `electrolyte`, Reach is `psi_*`. See the table in `docs/CODE_MAP.md`.
- Several data tables are fixed-order: rows may be renamed but not
  reordered, inserted or removed. `docs/NAMES.md` marks them.
- The draw-call buffer size in `src/gfx.rs` must stay at or below 65,535.
