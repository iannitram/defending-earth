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
# Job S0: the scale sheet, its Blender helper and the first size chart

## Goal

Make the one sheet of sizes that the game and every Blender script will
read, the small Blender helper every later model job uses, a soldier
stand-in, and a first size chart. Every later job depends on this one.

## The settled design

A battlefield square is 2 m across. A soldier is 1.8 m tall. Everything
else is declared in real units in one JSON file. There is only one copy.

Write `scale_sheet.json` with exactly these values (you may add a
`"note"` text to any entry, nothing else):

```json
{
  "version": 1,
  "anchor": { "square_m": 2.0, "soldier_height_m": 1.8 },
  "derived": {
    "soldier_shoulder_width_m": 0.55,
    "soldier_with_kit_width_m": 0.70,
    "door_width_m": 1.0,
    "door_height_m": 2.1,
    "low_cover_height_m": 1.0,
    "full_cover_height_m": 1.9,
    "ceiling_height_m": 2.6
  },
  "things": {
    "soldier": { "footprint": [1, 1], "height_m": 1.8, "width_m": 0.70, "depth_m": 0.45 },
    "regent":  { "footprint": [1, 1], "height_m": 3.6, "width_m": 1.8, "depth_m": 1.8 },
    "tender":  { "footprint": [1, 1], "height_m": 1.1, "width_m": 0.6, "depth_m": 0.6 },
    "lash":    { "footprint": [1, 2], "height_m": 1.7, "width_m": 1.9, "depth_m": 3.5,
                 "trunk_length_m": 3.5, "back_height_m": 0.7, "rifle_length_m": 2.6 },
    "gorger":  { "footprint": [1, 5], "footprint_head_out": [1, 1], "length_m": 10.0,
                 "body_diameter_m": 1.1, "tail_diameter_m": 0.5, "height_out_of_ground_m": 3.0,
                 "sign_patch_m": 2.0, "sign_heave_m": 0.04 },
    "kite":    { "footprint": [1, 1], "span_m": 1.8, "length_m": 1.5, "flight_height_m": 6.0 },
    "bloat":   { "footprint": [1, 1], "diameter_m": 1.5, "underside_height_m": 1.2, "feeler_length_m": 0.8 },
    "maul":    { "footprint": [2, 2], "height_m": 3.5, "width_m": 3.6, "depth_m": 3.8,
                 "spike_proud_m": 1.0, "spike_side_m": 0.22, "spike_collar_diameter_m": 0.7 },
    "cinder":  { "footprint": [1, 1], "sizes_m": [0.3, 0.6, 1.0] },
    "taker":   { "footprint": [1, 1], "closed_size_m": 1.2, "core_cube_m": 0.4,
                 "ring_diameter_m": 1.4, "hover_height_m": 1.0, "scout_span_m": 1.8, "scout_hover_height_m": 4.0 },
    "builder": { "footprint": [2, 2], "spread_m": 3.6, "leg_length_m": 2.6,
                 "core_sphere_m": 0.5, "closed_diameter_m": 1.8 }
  },
  "cheats": [
    "The Lash's rifle barrel may reach past its two squares. It blocks nothing.",
    "The Kite and the Bloat hang in the air above the one square they occupy.",
    "A buried Gorger covers no square. Head out it covers one, on the surface five.",
    "The taker's turning ring segments may reach 0.1 m past its square.",
    "The Regent, Tender and soldier stand inside one square with room to spare."
  ],
  "not_yet": [
    "Weapon ranges in metres, throw distance and Moon jump height: worked out when the rules are moved to 2 m squares.",
    "Vehicles, craft, base modules and buildings: added in their own phases."
  ]
}
```

`scale_lib.py` is a plain Python module for use inside Blender. It must
offer these, and nothing here is optional:

- `load()` returns the sheet as a dictionary. `square()` returns 2.0.
  `thing(name)` returns one entry and raises a clear error for a missing
  name or key.
- `reset_scene()`: empty scene, metres, Cycles on the CPU, 1600 by 900,
  64 samples, denoising, AgX view transform.
- `make_ground(size_m=60)`: a flat ground of dry soil, colour about
  `#6b5f4e`, broken up with two scales of noise so it does not tile, with
  a faint darker line (about 2 cm wide) every 2 m that shows the squares.
- `footprint_outline(w, d)`: a thin white line on the ground round a
  footprint of w by d squares, centred on the origin.
- `day_light()`: a physical sky, sun 38 degrees above the horizon, coming
  from the front left of a model that faces −Y, so shapes have a clear lit
  side and a shadow side. `night_light()`: nearly black sky with a weak
  cool moon from the same side, dim enough that the model's own lights
  are the picture.
- `add_soldier(x=0, y=0, facing_deg=0)`: a stand-in soldier 1.8 m tall and
  0.55 m across the shoulders. A plain mid-grey mannequin made of rounded
  limbs, with a head, torso, arms and legs in a relaxed standing pose.
  Not a box, and no face.
- `scale_bar(x, y, length_m=1.0)`: a flat black-and-white bar on the
  ground.
- Camera helpers that frame a list of objects: `cam_front`, `cam_side`,
  `cam_threequarter`, `cam_aerial` (45 m away, 40 degrees down, 135 mm
  lens), `cam_ortho_side`, `cam_detail(target, distance)`.
- `render(path)`, `contact_sheet(paths, out_path)` (tiles pictures with
  their file names underneath; use what Blender's own Python has, such as
  numpy and the image API), `measure(objects)` (width, depth, height and
  triangle count after modifiers) and `write_size_json(path, name,
  objects)`.

`size_chart.py` uses the helper to draw the first size chart: one row, in
side view with no perspective, of the soldier stand-in next to a plain
grey block for each of the ten creatures at its declared size (the Gorger
lying down at 10 m, the Kite and the Bloat at their heights in the air),
each with its name underneath and a 1 m scale bar. Later jobs replace the
blocks with real models; write the script so it uses
`source_assets/blender/creatures/<name>/model.blend` when that file
exists and the grey block when it does not.

## Files you may write

Only inside `source_assets/scale/`:
`scale_sheet.json`, `scale_lib.py`, `size_chart.py`, `README.md` (how to
use the helper, in plain words), `rounds.md`, and under `renders/`:
`size_chart.png`, `soldier_standin.png` (three-quarter view on the
ground, day light), `ground_day.png` and `ground_night.png` (the aerial
camera over empty ground with one soldier stand-in, to prove the ground,
the squares and both lights).

## Done looks like

- All four pictures exist and you have looked at each one.
- The squares are visible but quiet. The soldier stand-in reads as a
  person at the aerial camera distance. The night picture is dark but the
  soldier is just visible.
- Running `size_chart.py` twice gives the same picture.
- At least three judging rounds recorded in `rounds.md`.

## Must not touch

Anything outside `source_assets/scale/`. No game code. The game is changed
to read this sheet in a later job.
