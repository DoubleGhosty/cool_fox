# DexNom/VIctory launch flags in this tree

```powershell
cargo build --release --locked -p victory
$env:VICTORY_AMBIENT_FALLBACK = '1'
.\Play-VIctory.cmd --rt --sparse-textures --no-upload-ahead
```

| DexNom | Here |
| --- | --- |
| `--rt` | `VICTORY_BVH_BUILDER`, `VICTORY_ACTIVATE_RAYS`, `VICTORY_RT_RESOLVE` = 1 |
| `--no-rt` | the three = 0. Unlike DexNom, traversal and ray binning still run |
| `--rt-static-bvh` | builder 0, activation and resolve 1, `VICTORY_AMBIENT=0` |
| `--sparse-textures` | `VICTORY_LEVELS_ON_DEMAND=1` |
| `--upload-ahead`, `--no-upload-ahead` | `VICTORY_UPLOAD_AHEAD` = 1, 0 |
| `--warp` | `VICTORY_D3D12_WARP=1` (diagnostics builds) |
| `--console` | already here |
| `VICTORY_RAY_TRACING` = 0, 1, static-bvh | as the three flags; a flag wins |
| `VICTORY_AMBIENT_FALLBACK` | 1: `VICTORY_AMBIENT=constant:0.8` if unset; 0: off. This tree's stand-in light, not DexNom's material fill |

Accepted but **not implemented** (a warning, no effect): `--local-gpu-memory`,
`--fsr-quality`, `--fsr-balanced`, `--fsr-performance`, `--native-1440p`,
`--1080p-to-1440p`, `--capture-readbacks`, `--rt-write-stats`,
`--rt-store-trace`, `--lighting-trace`, `VICTORY_FSR_4K`.
Other DexNom-only `VICTORY_...` switches are not read at all.
