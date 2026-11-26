# VIctory on an RX 9070 XT

## Run it on Windows (RX 9070 XT)

This tree targets a 16 GB Radeon. The old `pareto-residency` launcher is the RTX 5090 residency build: it was happy holding about 18 GB of GPU memory, which does not fit a 9070 XT. Build on the Windows machine with the card. The prebuilt `victory-3d-rtx-pareto-residency.exe` is not this path.

You need Windows 10 or 11 x64, an AVX2 CPU, the pinned Rust toolchain, the MSVC C++ build tools with a Windows SDK, and your own dump of the title. The 9070 XT has mesh shaders and shader model 6.6, which the title needs. A Ryzen iGPU in the same PC is skipped.

```powershell
cd cool_fox
cargo build --release --locked -p victory
$env:VICTORY_GAME_DIR = 'D:\Games\MyDump'
.\Play-VIctory.cmd
```

Use this tree on the Windows PC. Cloning `DoubleGhosty/cool_fox` again gets the unmodified mirror, which is still the RTX launcher.

`Play-VIctory.cmd` with no argument is the 9070 profile:

- `VICTORY_GPU=amd` so the device is the Radeon, not an iGPU and not a leftover NVIDIA card. If two Radeons are installed, set `VICTORY_D3D12_ADAPTER=9070`.
- Series S, which is the memory profile that fits 16 GB. `Play-VIctory.cmd 9070-x` is Series X and wants about 64 GB of system RAM.
- A shader cache of its own at `%LOCALAPPDATA%\VIctory\cache-rdna4`, not the RTX cache. The first run stutters while the driver builds pipelines. `target\release\victory.exe warm --used` does that ahead of time.
- Textures are kept to about 82% of the card's Windows budget, empty texture blocks stay at 1 GB instead of 4 GB, and mip levels are uploaded as they are needed. The RTX knob that pinned every small buffer on the card is not set.
- Console shaders stay wave64. RDNA4 runs that natively. The wave32 barrier form that was only checked on RTX stays off, because it reset the card.
- The DXIL clean-up that reuses one instruction for one value stays off (`VICTORY_SHADER_OPT=59`). Adrenalin's compiler faults on that form on an RX 9070 XT (`amdxc64.dll+0x861b9f`) and does not on an RX 7900 XTX. A buffer size query also gets its own descriptor handle, which is the other form that compiler faults on.

`scripts\play.ps1 -Exe <path> -Gpu amd` is the same profile. `-RayTracing` turns the title's ray-traced indirect light on (slower). `-Safe` drops the newest scheduling switches when something draws wrong.

Every `victory run --execute` precompiles the shaders in the game folder into the cache, then loads the host pipelines recorded on earlier launches, before the title starts. The first launch is the long one. Later launches reuse the cache and only compile pipelines that are new. `VICTORY_PRELOAD=0` skips both. The pipeline book is on unless `VICTORY_PIPELINE_BOOK=0`. Preload uses four threads (`VICTORY_PRELOAD_JOBS`).

A 32 GB PC is the size this profile expects. The frame path stays the measured one: eight joined parts, and 192 lists in the air (128 was the count that still stopped the submitting thread). What is not in the current frame is released. Idle command lists nobody has taken for five seconds go, so a streaming spike does not leave its allocators behind. The pipeline library leaves the working set two seconds after a burst of loads, and stays in Windows' file cache. Texture staging that nothing has used for two seconds is cut back to 1.5 GB. A machine with more than 40 GB keeps the old sizes unless those variables are set.

`Play-VIctory.cmd pareto-residency` is still the RTX path.

The sections below describe the earlier RX 7900 XTX fork this code grew from. The 16 GB budgets above are what changed for the 9070 XT; the 7900 XTX (24 GB) still uses the wider limits when the card's budget is 20 GB or more.

A fork of [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer
that runs an Xbox Series title natively on Windows from files the user supplies.
This one is about **speed on AMD RDNA**: the world at 30 frames a second on
one PC, without giving up the picture. It is not there yet.
# cool_fox (CGBJ Runtime Continuation)

[![CI](https://github.com/DoubleGhosty/cool_fox/actions/workflows/ci.yml/badge.svg)](https://github.com/DoubleGhosty/cool_fox/actions/workflows/ci.yml)
[![PortWatch VI](https://img.shields.io/badge/PortWatch%20VI-91.5%25%20Playable-brightgreen)](https://portwatch-vi.xyz/#effort-cgbj)

Active community continuation and development branch of `cool_fox` (the CGBJ runtime, ranked **#1 / 91.5% Playable** on PortWatch VI).

This build unites the cutting-edge optimizations from leading PortWatch projects (`avatarmanz`, `codec-d`, `DexNom`, `Belobug`, and community contributors) into a unified, verified runtime for both AMD Radeon and NVIDIA GeForce GPUs.

---

## Key Features & Improvements

- **Pareto Memory Residency & 32 GB RAM Support:** Adaptive GPU memory residency limits, preventing WDDM paging exhaustion (`0x8007000E`) on mid-range and 32 GB systems.
- **Shader LOD Slot Fix (PR #2):** Leaves unused LOD slots undefined in array texture coordinate queries, resolving DXIL validator errors on retail shaders.
- **NVIDIA & Video Stream Synchronization (PR #6):** Synchronous flush for decoded video frame copies, eliminating stutter and repeated frames in opening videos during multithreaded recording (`VICTORY_THIN=1`), and constant ambient lighting (`VICTORY_AMBIENT=constant:0.8`).
- **Wine / Proton & DXIL Robustness (PR #10):** Code-generated root signatures replacing fragile comment pragmas for vkd3d-shader, single-latch DXIL loop CFG normalization, and 256 MiB pipeline worker stack.
- **Voxel Visibility Recovery:** Restores wave64 mask provenance when restored from scalar copies, enabling foliage and volumetric voxel visibility shaders to pass compilation.
- **Custom Resolution & Scaling:** `VICTORY_OUTPUT=1920x1080` (or `2560x1440`) and `VICTORY_RENDER_SCALE=0.667..1.0` with the 4K backbuffer pool check bypass.
- **Frame Rate Unlock:** `VICTORY_FRAME_INTERVAL_MS=16.667` (60 FPS) or `8.333` (120 FPS) while keeping game simulation clock strictly at 1.00x.
- **Renderer Port 5 (Zero-Copy CBV):** `VICTORY_PORT_CBV=1` reads constant buffers as raw bytes in-place, eliminating ~22,000 buffer copies per frame.

---

## How to Run

You need Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with shader model 6.6 and mesh shaders, Rust (1.98+), MSVC build tools, and your own legal Xbox Series dump of the game.

### Build from Source

```powershell
git clone https://github.com/DoubleGhosty/cool_fox.git
cd cool_fox
cargo build --release --locked -p victory
```

### Launch

```powershell
$env:VICTORY_GAME_DIR = 'D:\Games\MyDump'
.\Play-VIctory.cmd
```

For 4GB / 8GB VRAM laptops or low-memory systems, you can also launch with:

```powershell
.\Play-VIctory-4GB.cmd
```

---

## Upstream & Acknowledgements

- **codec-d / CGBJ:** Original `cool_fox` runtime and pareto residency architecture.
- **AvatarManz (VIctory-7900XTX):** Multi-threaded command list recording, framerate unlocking, and renderer ports.
- **DexNom:** Wave64 lowering, scalar recovery, and shader structure passes.
- **Belobug:** Ambient lighting and integer target blend protections.
- **Marvisak & fxceless:** Foundational VIctory compatibility layer.

**Legal notice:** This repository contains no copyrighted game code, assets, or executables. All code is licensed under GPL-3.0-or-later.
copy victory.example.toml victory.toml      # then set your paths, outside this folder
cargo build --locked --profile diag -p victory --features diagnostics
scripts\play.ps1 -Exe <path of your copy's executable>
```

`scripts\play.ps1 -Safe` leaves out the newest, faster ways; try it first when
something is wrong. `-RayTracing` turns the title's ray-traced indirect light on.

Before the first run, ahead of time (both can be stopped and go on later):

```powershell
target\diag\victory.exe shaders <the shader archives>   # every shader translated, minutes
target\diag\victory.exe warm --used                     # pipelines earlier runs drew with, for a new driver or build
```

Commands, debugging switches and upstream's own description:
[docs/README-fxceless.md](docs/README-fxceless.md). Every `VICTORY_...` switch is
described in a comment where it is read.

## What is different here

In short; the reasons and the measurements are in [docs/findings.md](docs/findings.md).

- **List calls are written down, not carried out, on the title's threads**
  (`VICTORY_THIN`): eight threads of ours translate them.
- **Submissions are prepared by a thread of ours and joined**
  (`VICTORY_ASYNC_SUBMIT`, `VICTORY_JOIN_PARTS`), with what can be spread over
  threads spread: refreshes, compares (AVX2), descriptor records, landings.
- **What shaders wrote comes back beside the write guards**, by the dwords that
  changed, on a thread of its own (`VICTORY_RELAY`, `VICTORY_LAND`).
- **Buffer fills that need bytes still on the card are done on the card**
  (`VICTORY_CARD_STEPS`) instead of waiting for it.
- **The title's writes are caught more cheaply**: one view opened at a fault,
  blocks that rest, places in the title's code that look before they write.
- **Textures go up on a copy queue of their own** and a slot shows one only after
  its upload ran, which also ended a device loss that came every 15 minutes or so.
- **Boats float**: textures that shaders write and the title's CPU reads go back
  to title memory (from fxceless and Geckomacabre, on by default here).
- **Frame record and replay** with per-pass times and shader variants compared
  turn by turn ([docs/replay.md](docs/replay.md)).

## Credits

This is a tree of other people's work with more on top. The history is kept, and
commits taken from elsewhere name the commit and its author.

- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and its
  contributors: the loader, the runtime services, the Direct3D 12 back end and
  the shader recompiler.
- **[fxceless/victory-fork](https://github.com/fxceless/victory-fork)**: the fork
  this one started from, and the people whose pull requests it carries.
- **[DexNom/VIctory](https://github.com/DexNom/VIctory)**: ray tracing fixes,
  cube gradients, sparse textures.
- **[Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD)**:
  the ambient light idea, pipeline workers, and work by romainhedouin.
- **[FQY55kai/ZJOU-VICTORY](https://github.com/FQY55kai/ZJOU-VICTORY)**.
- **[myokex/victory-amd](https://github.com/myokex/victory-amd)**: the guard over
  every view of a page (with Spexivus, joroyt and tightfall), and Joshua Odom's
  reads into title memory.
- **Geckomacabre** and **cxy0072019-sketch**: the texture read-back.
- **[XCL](https://github.com/ivall/xcl)** (GPL-2.0-only) was read for how the same
  title behaves under another layer; no code of it is here.
- This fork's changes were made with [Claude Code](https://claude.com/claude-code).

## License

GPL-3.0-or-later, as VIctory. See [LICENSE](LICENSE); the one file that is not
covered is named in [docs/asset-policy.md](docs/asset-policy.md).
