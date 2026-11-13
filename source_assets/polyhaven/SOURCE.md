# Poly Haven textures

Downloaded 2026-11-13 from https://polyhaven.com (file host dl.polyhaven.org), 1k JPG.
Licence: CC0 1.0 (public domain) for every asset on Poly Haven.

Per asset: `_diff` colour, `_nor_gl` normal map (OpenGL), `_arm` ambient occlusion / roughness / metal in R/G/B.

| Folder | Page |
|---|---|
| leafy_grass | https://polyhaven.com/a/leafy_grass |
| sparse_grass | https://polyhaven.com/a/sparse_grass |
| farm_furrows | https://polyhaven.com/a/farm_furrows |
| farm_soil | https://polyhaven.com/a/farm_soil |
| dry_mud_field_001 | https://polyhaven.com/a/dry_mud_field_001 |
| gravel_road | https://polyhaven.com/a/gravel_road |

Packed for the game by `make_ground.py` into `assets/ground/<name>.png`
(512², R = brightness grain, G/B = normal x/y, A = roughness). `farm_furrows`
is kept but unused. `gravel_road` ships an empty roughness channel; the
script substitutes a constant.
