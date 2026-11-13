# CREDITS AND SOURCES

## Code and design

Game design, names, numbers and code are this project's own. The rules are
written down in `MECHANICS.md`, the names in `docs/NAMES.md`, and the visual
direction in `docs/ART_BIBLE.md`. Soldiers, creatures, terrain, effects, UI and
sound are generated at runtime by the Rust/macroquad code in `src/`.

## Public-domain geography

* **Natural Earth 1:50m land** (naturalearthdata.com, public domain) — coastlines
  used to build `assets/earth_data.png`.
* **Natural Earth 1:50m lakes** (public domain) — lake holes in the land mask.
* **Natural Earth I with shaded relief and water, 1:50m** (public domain) —
  source of the 4096×2048 albedo `assets/earth_albedo.png`.
* Made with Natural Earth. The data is in the public domain; no attribution is
  required, but it is given here.

## Other public-domain sources

* **NASA Earth Observatory "Earth at Night" (Black Marble)** global map, 2012
  edition (NASA imagery, public domain) — source of the city-lights image
  `assets/earth_lights.png`. The copy in `source_assets/nasa_black_marble/`
  was fetched from Wikimedia Commons (File:The_earth_at_night.jpg). See
  `source_assets/nasa_black_marble/SOURCE.md`.
* **Poly Haven** ground textures (polyhaven.com, CC0 1.0, free for any use
  with no credit required): leafy_grass, sparse_grass, farm_soil,
  dry_mud_field_001, gravel_road — packed into `assets/ground/*.png`. A sixth
  download, farm_furrows, is kept in `source_assets/polyhaven/` but is not
  used or shipped. See `source_assets/polyhaven/SOURCE.md`.
* Sky colours in `assets/sky/looks.txt` are measured from this project's own
  Blender sky renders in `source_assets/blender/sky/`. No outside sky images
  are used.

## Fonts

* `assets/ui.ttf` and `assets/ui_bold.ttf` are DejaVu Sans and DejaVu Sans Bold.
  DejaVu fonts are based on Bitstream Vera and are distributed under the
  Bitstream Vera license, which permits free use and redistribution. The
  license text is available at https://dejavu-fonts.github.io/License.html.
