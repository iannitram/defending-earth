# The scale sheet and its Blender helper

This folder holds the one sheet of sizes for Defending Earth and the small
helper that every Blender model job uses.

- `scale_sheet.json`: every size, in metres. There is only one copy.
- `scale_lib.py`: the helper. It reads the sheet and makes the shared
  ground, light, soldier stand-in, cameras and check pictures.
- `size_chart.py`: draws the size chart and the pictures that prove the
  helper works.
- `renders/`: those pictures.
- `rounds.md`: notes from the judging rounds.

## The two anchors

A battlefield square is 2 m across. A soldier is 1.8 m tall. Everything
else in the sheet is declared in real units next to those two.

## Rules

- Never type a size into a model script. Ask the sheet:
  `sl.thing("maul")["height_m"]`.
- If the size you need is not in the sheet, stop and ask the lead. The
  helper raises a clear error that names the missing key.
- Do not copy the sheet. Do not edit it in a model job.

## Running Blender

Blender is the Flatpak build. It runs in the background only. Give it the
full path of the script. A path that starts at the project folder does not
work, because the Flatpak starts in its own folder.

```bash
cd "/home/deck/dev/Defending Earth"
flatpak run org.blender.Blender -b --python "$PWD/source_assets/scale/size_chart.py"
```

That writes all four pictures (about 45 seconds). To draw only some, name
them after `--`: `chart`, `soldier`, `ground`. Add `--out <folder>` to
write somewhere else.

The Flatpak cannot read `/tmp`. Keep scratch files under `/home/deck`.

## Using the helper in a model script

```python
import sys
sys.dont_write_bytecode = True   # leaves no __pycache__ folder in the project
sys.path.insert(0, "/home/deck/dev/Defending Earth/source_assets/scale")
import scale_lib as sl

sl.reset_scene()                       # empty scene, standard render settings
maul = sl.thing("maul")                # sizes from the sheet
sl.make_ground(footprint=maul["footprint"])
sl.day_light()
# ... build or load the model, collect its objects in a list `parts` ...
sl.footprint_outline(*maul["footprint"])
sl.cam_aerial(parts)
sl.render("/home/deck/.../renders/04_aerial.png")
```

### What each call does

| Call | What you get |
|---|---|
| `load()` | The whole sheet as a dictionary. |
| `square()` | 2.0, the width of one square in metres. |
| `thing(name)` | One entry of the sheet. A wrong name or key gives a clear error. |
| `derived(key)` | One value from the sheet's `derived` list (doors, cover heights). |
| `reset_scene()` | Empty scene in metres. Cycles on the CPU, 1600 by 900, 64 samples, denoising, AgX. |
| `make_ground(size_m=60, footprint=(1, 1))` | Dry soil with a faint darker line every 2 m. |
| `footprint_outline(w, d)` | Thin white line round a footprint of w by d squares, centred on the origin. |
| `day_light()` | Physical sky. Sun 38 degrees up, from the front left. |
| `night_light()` | Nearly black sky and a weak cool moon from the same side. |
| `add_soldier(x, y, facing_deg=0)` | The grey soldier stand-in, 1.8 m tall. |
| `scale_bar(x, y, length_m=1.0, axis="Y")` | Black-and-white bar on the ground, 0.25 m per segment. |
| `cam_front(objects)` | Camera in front, at mid height, everything in frame. |
| `cam_side(objects)` | Camera on the lit left side. The model faces right in the picture. |
| `cam_threequarter(objects)` | Camera to the front right. Shows the lit front and the shadow side. |
| `cam_aerial(objects)` | The game's view: 45 m away, 40 degrees down, 135 mm lens. |
| `cam_ortho_side(objects)` | Side view with no perspective, for the size picture. |
| `cam_detail(target, distance)` | Close view of an object or of a point `(x, y, z)`. |
| `render(path)` | Renders the current camera to a PNG. |
| `contact_sheet(paths, out_path)` | All pictures on one sheet, file names underneath. |
| `measure(objects)` | Width (X), depth (Y), height (Z) and triangle count, after modifiers. |
| `write_size_json(path, name, objects)` | Measures the model and writes its `size.json`. |
| `rgb("#rrggbb")` | A hex colour as the linear colour Blender nodes need. |
| `new_material(name, hex, roughness, ...)` | A plain Principled BSDF material. |
| `bounds(objects)` | The box round the objects, as two corner points. |

### Things to know

- **Directions.** The model faces −Y. Its front is toward Blender's front
  view. The sun and the moon stand over the −X, −Y quarter. Seen from the
  front they are on the left. The three-quarter and aerial cameras stand to
  the front right, 35 degrees round from straight ahead.
- **The ground lines follow the footprint.** A model 1 square wide stands
  in the middle of a square. A model 2 squares wide stands on a line. Pass
  the model's footprint to `make_ground` so the lines and the white outline
  agree.
- **The soil goes on past the squares.** The lines cover `size_m` by
  `size_m`. The plain soil carries on for 2 km each way, so a level camera
  sees a true horizon and not the edge of a plate.
- **Children count.** The cameras, `measure` and `write_size_json` take the
  objects you list and all their children. Listing the root is enough.
- **Cameras take the place of older ones.** Calling `cam_front` twice
  leaves one front camera. The last camera made is the one that renders.
- **`footprint_squares` in `size.json`** is the footprint the sheet
  declares. If the measured model is bigger than that, a warning is
  printed. Check the sheet's `cheats` list before worrying: some overhang
  is allowed.
- **The scale bar is 5 cm high**, so the side cameras can see it. For the
  front camera pass `axis="X"`.
- **Day and night in one script.** Call `day_light()`, render, call
  `night_light()`, render. Each call clears the other.
- **The stand-in** is 1.8 m tall and 0.55 m across the shoulders, about
  0.63 m across the hanging arms and 0.30 m deep. It has about 12,800
  triangles. It has no face and no kit.

## The size chart

`size_chart.py` draws one row, seen from the side with no perspective: the
soldier stand-in, then each of the ten creatures. Under each is its name,
one key size, and a white mark that shows how many squares it takes up
along the row. The lines on the wall are 1 m apart.

A creature is a plain grey block until its real model exists. As soon as
`source_assets/blender/creatures/<name>/model.blend` is there, the chart
uses it. If that file has a collection called `model`, only that
collection is brought in. Otherwise every object is, except cameras,
lights and the shared stand-ins. A flyer (Kite, Bloat, taker) that was
built standing on the ground is raised to its height from the sheet.

Three things on the chart are not in the sheet. They only shape the grey
blocks and are waiting for the lead:

- The Kite's block is 0.3 m thick. The sheet gives the Kite no thickness.
- The builder's block is as tall as its `closed_diameter_m` (1.8 m). The
  sheet gives the builder no standing height.
- The Kite's `flight_height_m` and the taker's `hover_height_m` are drawn
  as the height of the underside, the same way the Bloat's
  `underside_height_m` is.

Running the script twice gives the same picture, byte for byte.
