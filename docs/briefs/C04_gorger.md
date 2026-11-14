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
# Blender rules for this job

- Blender is the Flatpak build, version 5.3. There is no `blender` command.
  Run it only like this, from the project folder:
  `flatpak run org.blender.Blender -b --python <script.py> -- <arguments>`
  It can read and write anywhere under `/home/deck`.
- The Python API may differ from what you remember. When a call fails, look
  it up by introspection inside Blender (`dir()`, `bl_rna.properties`),
  do not guess twice.
- **Everything is built by your script.** One script, `build.py`, in your
  folder, builds the whole model from nothing and saves `model.blend`. A
  second script, `render.py`, opens `model.blend` and writes the check
  pictures. Running both again must give the same result.
- **Sizes come from the scale sheet**, never typed into your script. Read
  `source_assets/scale/scale_sheet.json` through the helper
  `source_assets/scale/scale_lib.py` (add its folder to `sys.path`). If a
  size you need is not in the sheet, that is an `UNDECIDED:` stop. (The
  job that makes the scale sheet is the one exception.)
- Units are metres. Z is up. The model stands on the ground at Z = 0, its
  footprint centred on the origin, facing −Y (toward Blender's front view).
- Geometry: meshes made by script (bmesh, primitives, skin and subdivision
  modifiers, curves, metaballs turned into mesh). Materials: Principled
  BSDF with procedural textures (noise, Voronoi, wave, gradients). No
  downloaded textures, no image files. Baking waits for a later job.
- Size limit: at most 40,000 triangles for one creature or figure after
  modifiers are applied, unless the brief gives another number. In the game
  it is seen 30 to 80 pixels tall, so the silhouette and the large shapes
  matter most; fine detail goes into the material.
- Every part gets a clear object name. Parts that will move later (limbs,
  jaws, plates, lenses) are separate objects parented in a sensible tree,
  with their origin at the joint. No rig, no animation.

## Shared colours (use these exact values)

| Thing | Colour | Surface |
|---|---|---|
| The frame under enemy flesh (the material SPINE) | `#14171a` | regular lattice with 6 cm cells, slightly wet, roughness 0.3 |
| Registrar hardware (spike, rings, cores, plates) | `#0b0b0c` | exact flat faces, sharp edges, roughness 0.35, metallic 1 |
| Etched measuring circles on Registrar plates | `#8a6a2a` | thin lines only, dull gold, never glowing |
| Amber lens or seam light (Registrar only) | `#ffa01e` | emission strength 6 |
| Electrical life (living creatures only) | `#cfe4ff` | tiny thin arcs at joints, emission 4; shown only in the night picture |
| The Glow (the taker's fluid) | `#c8e61e` | emission 5, wet |
| Magma and fire | `#ff5a0c`, white-hot `#fff2d0` at the deepest point | emission 8 to 30 |

Amber belongs only to Registrar hardware. Yellow-green belongs only to the
Glow. Orange belongs only to fire, the Maul and Cinders. Never mix them.
Nothing is glossy plastic, chrome or neon. No glowing outlines. No faces:
no eyes, mouths or expressions on any enemy.

## Check pictures (write these into your folder's `renders/`)

Render with Cycles on the CPU, 1600 by 900, 64 samples, denoising on, view
transform AgX. Keep any one picture under about three minutes; lower the
samples before lowering the size. Use `scale_lib.py` for the ground, the
light and the soldier stand-in so every job's pictures match:
- Ground: `scale_lib.make_ground()` (dry soil with faint lines every 2 m
  that show the battlefield squares).
- Day light: `scale_lib.day_light()`. Night: `scale_lib.night_light()`.
- Soldier stand-in, 1.8 m: `scale_lib.add_soldier(x, y)`.

Pictures every model job must write, unless the brief changes the list:
1. `01_front.png`, `02_side.png`, `03_threequarter.png`: day light, camera
   at the model's mid height, the whole model in frame.
2. `04_aerial.png`: the game's view. Camera 45 m away, looking down at 40
   degrees, 135 mm lens, day light. The footprint squares are drawn on the
   ground in a thin white line.
3. `05_size.png`: straight side view, no perspective (orthographic), with
   the soldier stand-in beside the model and a 1 m scale bar.
4. `06_night.png`: three-quarter view, night light, so only the model's
   own lights and a little moonlight show.
5. `07_detail.png`: a close view of the part named in the brief.
6. `08_dead.png`: the dead or wrecked form, three-quarter view, day light.
7. `contact.png`: all of the above on one sheet with their names.

## Judging (you do this before you finish)

Work in at least three rounds. In each round: render, **open and look at
every picture** with the Read tool, judge it as a harsh art director would
judge a still from aerial news footage, write down the three weakest
things, fix them, render again. Keep the notes in `rounds.md` in your
folder: per round, the three weakest things and what you changed. Typical
failures to look for: a shape that reads as a toy or as smooth plastic;
parts that float or pass through each other; sizes that do not match the
sheet; a silhouette that is unclear from the aerial view; lights that are
too even or too bright; anything that looks like a face.

## Other files every model job writes in its folder

- `sheet.md`: the design sheet in plain words. What it is, what it is for,
  how it looks, its parts, its sizes (quoted from the scale sheet), its
  colours, its lights, its dead form, and what was left for later (rig,
  motions, baked textures).
- `size.json`: measured from the finished model by your script:
  `{"name": ..., "width_m": ..., "depth_m": ..., "height_m": ...,
  "footprint_squares": [w, d], "triangles": ...}`. Width is along X, depth
  along Y.

## Added after the trial run (these override anything above that differs)

- **Starting Blender.** The Flatpak starts in its own folder and cannot
  read `/tmp`. Always give the script's full path, and keep every file
  you need inside your own folder in the project:
  `flatpak run org.blender.Blender -b --python "/home/deck/dev/Defending Earth/<your folder>/<script>.py"`.
  Delete any `__pycache__` folder Blender leaves behind.
- **The helper library exists and is finished:**
  `source_assets/scale/scale_lib.py`. Read its `README.md` first and use
  its calls as they are. Do not edit anything in `source_assets/scale/`.
  `make_ground` takes `footprint=(w, d)` so the square lines sit on the
  edges of your model's footprint.
- **Sun side.** "Front left" means the viewer's left when looking at the
  model's front: the sun is over the −X, −Y quarter.
- **Heights of things in the air.** `flight_height_m`, `hover_height_m`
  and `underside_height_m` in the sheet all mean the height of the
  lowest point of the body above the ground (hanging feelers and the
  like do not count). Build the model in place at that height.
- **Sizes the sheet does not give** (a thickness, a standing height) come
  from your job's own text below. If your job's text does not give one
  either, choose nothing: stop with `UNDECIDED:`.
# Job C04: the Gorger and its buried signs (design sheet, model, check pictures)

## Goal

Build the Gorger as a Blender model at true size, in three states, with
its design sheet and check pictures. Folder:
`source_assets/blender/creatures/gorger/`. Sheet name: `gorger`.

## The settled design

**What it is.** An animal. A long armoured worm that lies buried, feels
footsteps through the ground, bursts out, seizes a victim in its jaws and
drags the remains below. It makes open ground frightening. Its meat is
food for freed aliens, so its body is also something the player wants.

**Overall shape.** A worm 10 m long, 1.1 m thick at its thickest (about a
third of the way back from the head), tapering to 0.5 m at the tail.

**Parts.**
- *Plates:* 28 overlapping armour plates along the back and sides, pale
  and scuffed like local stone (`#a39c8c`, matte, chipped edges, soil in
  the gaps). Between and under them dark red flesh (`#5a1c1c`, damp). The
  belly is flesh with the dark frame showing as regular ribs.
- *Mouth:* six curved jaws, each 0.9 m long, that open outward like a
  flower. Inner faces dark red, lined with short barbs. No eyes.
- *Feelers:* ten thin spines 1.2 m long in a ring just behind the jaws,
  swept back.
- *Control hardware:* just behind the feelers, on the top, a cluster of
  five amber lenses in a black housing, and one broken black arc of the
  Registrar's ring (0.6 m long) bolted across the plates.

**Wrong repair.** Three plates in a row, about half way along, are flat
black Registrar plate instead of stone-coloured armour.

**Electrical life.** Faint pale arcs between the feelers, in the night
picture only.

**Three states, built as three collections in the one file:**
1. *On the surface:* the whole body lying in a gentle S-curve that stays
   inside a strip one square (2 m) wide and five squares (10 m) long.
   Jaws half open.
2. *Head out:* the head and the first 3.0 m of body rising straight out
   of the ground inside one square, jaws fully open, with a ring of
   thrown-up soil and broken clods around the hole. The rest of the body
   is not built in this state.
3. *Buried sign:* nothing of the animal is visible. A patch of ground one
   square (2 m) across where the soil is raised 4 cm in a low uneven
   mound, with a few fine cracks from the centre and slightly lighter,
   looser soil. One thin wisp of dust, about 0.4 m tall, leaning with the
   wind, rising from one crack. **It must be faint.** A player who is not
   looking should miss it at the aerial camera distance; a player who is
   looking should find it.

**Dead form.** State 1 with the jaws slack and fallen open, feelers limp,
lenses dark, and one plate cracked through.

**Pictures.** Use the standard list for state 1 (the surface form), with
these changes and additions:
- `06_night.png`: state 2 (head out) at night.
- `07_detail.png`: the open mouth and control hardware of state 2.
- `09_head_out.png`: state 2 three-quarter, day, with the soldier stand-in
  1.5 m away.
- `10_sign_aerial.png`: state 3 from the aerial camera, with one soldier
  stand-in three squares away, no outline drawn on the ground.
- `11_sign_close.png`: state 3 from 6 m away at standing eye height.
- `12_sign_compare.png`: the aerial view again with the patch circled in
  a thin red line, so the lead can find it.
Write `size_surface.json` (footprint 1 by 5) and `size_head_out.json`
(footprint 1 by 1) as well as `size.json` (same as the surface form).

## Files you may write

Only inside `source_assets/blender/creatures/gorger/`: `build.py`,
`render.py`, `model.blend`, `sheet.md`, the three size files,
`rounds.md`, and pictures under `renders/`. Triangle limit for the
surface form: 60,000.

## Done looks like

All pictures and the contact sheet exist. Length is 10 m within 5
percent. The buried sign is judged for faintness in each round and the
notes say what was changed. Three judging rounds are in `rounds.md`.

## Must not touch

Anything outside that folder.
