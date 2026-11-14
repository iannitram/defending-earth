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
# Job R1: the eleven ribbons, five levels each, and the ribbon board

## Goal

Make the pictures of every ribbon at every level, and of the ribbon
board, with a design sheet. Folder: `source_assets/ribbons/`.

## The settled design

**What it is.** Soldiers specialise by earning ribbons. Each ribbon has
five levels; a higher level replaces the lower one. Ribbons are worn on
the RIBBON BOARD on the dress uniform. The names below are final and are
the on-screen names.

**The bar.** Every ribbon is a woven cloth bar 35 mm wide and 10 mm
tall, drawn at 700 by 200 pixels with a clear (transparent) background
and 20 pixels of clear margin. The cloth has a fine ribbed weave running
top to bottom and a very slight softness at the edges. Colours are muted
and matte, like real ribbon cloth seen in daylight. Stripes run top to
bottom and are placed symmetrically unless stated.

**The eleven ribbons** (base colour, then stripes):

| Name | Base | Stripes |
|---|---|---|
| RIFLE | dark green `#3d4a36` | one thin sand `#b5a27a` stripe 3 mm in from each end |
| SHOTGUN | rust `#7a3b22` | one wide black `#1c1c1c` band in the centre, 9 mm |
| KNIFE | steel grey `#6d7378` | one thin dark red `#7d1f1f` line in the centre, 1.5 mm |
| EXPLOSIVES | black `#1c1c1c` | two burnt orange `#b8571c` bands, each 4 mm, 7 mm either side of centre |
| MEDIC | off-white `#cfc8b6` | one dark red `#6e1b1b` band in the centre, 7 mm |
| SCOUT | sand `#a8946a` | three thin dark green `#3d4a36` lines in the centre, 1.5 mm each, 1.5 mm apart |
| OVERWATCH | navy `#26324a` | one pale grey `#b9bcc0` stripe 2 mm wide at each end |
| REACH | deep violet `#3b2a4f` | one thin dull gold `#9a7b2e` line in the centre, 1.5 mm |
| SURVIVOR | dark grey `#3a3a3a` | one pale blue `#8fa3b0` stripe 5 mm wide at each end |
| CREATURES | brown-purple `#45303a` | two amber-brown `#b27a1e` lines, 2 mm each, 5 mm either side of centre |
| CHARMED | bottle green `#1f4a3a` | one silver-white `#c9c9c4` line 2 mm wide, off centre: 10 mm from the left end |

**The five levels: the same cloth with added metal.**
- Level 1: bare cloth.
- Level 2: one small stud in the centre.
- Level 3: two studs, 6 mm apart, centred.
- Level 4: three studs, 6 mm apart, centred.
- Level 5: three studs as level 4, plus a thin frame round the whole bar.
- A stud is a low dome 2.2 mm across of dull grey-brown metal
  (`#7d776c`), matte, with a soft small highlight and a tiny shadow on
  the cloth. The frame is the same metal, 0.6 mm wide, with square
  corners. Never shiny, never gold.

**The ribbon board.** A dark backing (`#23272a`, matte) holding bars in
rows of three with 1.5 mm between bars and between rows, 2 mm of
backing showing round the outside. Rows fill from the top left. A last
row with fewer than three bars is centred. The order on the board is the
order of the table above.

**How to draw them.** With a script, so the result can be made again.
Use Blender in the background (flat geometry seen straight on with no
perspective, soft even light from the upper left, clear background) or
Blender's bundled Python with numpy. Do not install anything.

**Pictures.**
- `bars/<name>_<level>.png` for all 55, names in lower case
  (`rifle_1.png` … `charmed_5.png`).
- `sheet_all.png`: a grid, eleven rows by five columns, with the name at
  the left of each row and the level number above each column, on a
  mid-grey background.
- `board_full.png`: a board with all eleven at level 1.
- `board_example_a.png`: RIFLE 4, MEDIC 2, SURVIVOR 1.
- `board_example_b.png`: OVERWATCH 5, SCOUT 3, RIFLE 2, CREATURES 2,
  CHARMED 1.
- `board_small.png`: `board_example_b` drawn 96 pixels wide, to check it
  still reads when small.

**Files to write besides the pictures.** `sheet.md` (the design in plain
words with the table, the level rule and the board rule),
`ribbons.json` (names, colours, stripe positions in millimetres, stud
and frame sizes), `build.py`, `rounds.md`.

## Files you may write

Only inside `source_assets/ribbons/`.

## Done looks like

All pictures exist and you have looked at every one. Any two ribbons can
be told apart at the small size. The five levels of one ribbon can be
told apart at the small size. Nothing looks like a flat coloured
rectangle: it reads as cloth with metal on it. Three judging rounds are
in `rounds.md`.

## Must not touch

Anything outside that folder. Do not copy the ribbon of any real medal or
country; if a design here turns out to match a well-known real one that
you recognise, say so in your report and change nothing.
