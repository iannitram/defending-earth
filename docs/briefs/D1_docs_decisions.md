# Rules for every job (read all of this first)

You are one run in a larger piece of work on the game Defending Earth
(Rust, macroquad). The project folder is `/home/deck/dev/Defending Earth`.
A lead has already made every design decision. You carry the brief out.

- **Do not ask questions.** Nobody is there to answer. If the brief does not
  cover a choice that changes what the result looks like, what it is called,
  or how a rule works: **stop**, change nothing further, and end with a
  report that starts with the line `UNDECIDED:` and states the question.
  Do not guess. Small technical choices (variable names, mesh density, how
  to call a Blender function) are yours.
- **Write only the files listed under "Files you may write".** Read anything
  in the project. Never write anywhere else in the project.
- **Never commit, push, stash, reset or otherwise touch git.**
- **Never launch the game** (`defending_earth`, `lightlab`). The game opens
  a window on the owner's screen. Only the lead launches it.
- **Never use a sub-agent, the Agent tool or a workflow.** Do the work
  yourself in this run.
- **Never use the Blender MCP tools or connect to an open Blender.** Blender
  runs only in the background from the shell (see the Blender section if
  your job has one).
- **Original work only.** Never name, copy or allude to an existing
  commercial game, film or real military unit. No real flags, badges,
  camouflage patterns, maker's names or real people's names or faces.
- **The repository is public.** Never write personal details, keys or
  passwords into any file.
- Plain words in every document you write: short sentences, no jargon
  without a one-line explanation. Times are written like 7am and 4:45pm.
- Pictures in `reference/` and `docs/lore_refs/` are idea references only.
  You may look at them. Build from the description in this brief, never
  copy a picture.

## How to finish

End with a short plain report, in this order:
1. `DONE`, `PARTLY DONE` or `UNDECIDED:` on the first line.
2. Every file you wrote, one per line.
3. What you checked and the result (commands run, numbers measured).
4. The three weakest things in the result, honestly.
5. Anything the lead must decide.
# Job D1: bring the documents in step with the tenth session's decisions

## Goal

The owner made a set of decisions today. They are recorded in
`docs/LORE.md` under "Tenth session (2026-11-13): the design fan-out"
(rounds 1 to 6). Other documents still say "to settle" for the same
things. Bring them in step. You change wording only; you decide nothing.

## The settled design

`docs/LORE.md`, tenth session, is the only source. Read all six rounds.
Where another document disagrees with it or says the matter is open,
change that document to say what was decided, in the same plain style,
with "(decided 2026-11-13, tenth session)". Do not copy whole
paragraphs; one or two sentences and a pointer to `LORE.md` is right.

Places to change (find the exact lines yourself):

- `docs/ROADMAP.md`
  - The scale sheet section: the anchor is settled at 2 m.
  - The catalogue: ribbons are eleven, five levels each, 55 designs (it
    says ten and 50); the names and the level rule are settled; the
    dress uniform and where it is seen are settled; the face slider
    list is 24 in six groups plus age and weight (final check when the
    head sheet is shown); the Cinder shows no Registrar marks; the
    Maul's spike is settled; buried Gorger signs are settled as faint;
    big creatures cover several squares, with the footprints, and the
    rule change needs its own plan; refugee harnesses wait until the
    three bodies are accepted.
  - Graphics phase 3: two flares (hand flare and sky flare), the weapon
    flashlight that comes on in the dark, the Glow is yellow-green; the
    three sky photographs are in this pass.
  - The numbered list "Still to decide": mark questions 26, 35, 39 and
    60 as answered, each with a one-line answer. Read each question
    before marking it; if a question's text does not match the answer
    in `LORE.md`, leave it and name it in your report.
  - New open items to add to that list, as questions: the sky flare's
    on-screen name, cost and numbers; whether a lit soldier is easier
    for the enemy to spot; the numbers for light steadying fear of the
    dark; the plan for creatures that cover several squares.
- `docs/ART_BIBLE.md`, section 5: the "to settle" notes on Cinder marks,
  the Maul's spike, buried Gorger signs and refugee harnesses; the six
  portrait stages ("not designed yet") now have their one-line
  descriptions; the Glow's colour; the flashlight rule in section 4
  ("suit lights") now says the light is on the weapon.
- `docs/NAMES.md`: add the eleven ribbon names (RIFLE, SHOTGUN, KNIFE,
  EXPLOSIVES, MEDIC, SCOUT, OVERWATCH, REACH, SURVIVOR, CREATURES,
  CHARMED) and RIBBON BOARD, CITATION and BOOK OF CITATIONS as final
  on-screen names, marked "decided, not yet in the game", in the place
  and style the file already uses for decided names. Do not reorder,
  insert into or remove from any table the file marks as fixed-order.
- `docs/STATUS.md`: under what is next, a short note that the design
  pass of the tenth session is under way, what it covers, and that
  nothing from it is in the game yet.
- `docs/LORE.md`: only the "Still to decide" section near the end.
  Remove or mark as answered the items the tenth session answered.
  Change nothing else in this file.

## Files you may write

`docs/ROADMAP.md`, `docs/ART_BIBLE.md`, `docs/NAMES.md`,
`docs/STATUS.md`, and the "Still to decide" section of `docs/LORE.md`.

## Done looks like

Searching those four documents for "to settle" finds no item that the
tenth session settled. The report lists each change as: file, the old
sentence, the new sentence.

## Must not touch

`docs/CODE_MAP.md`, `AGENTS.md`, `MECHANICS.md`, `mechanics.json`, any
code, any fixed-order table's order, and any decision's meaning.
