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

**This repository contains no game files, code or assets.** You need your own
copy of the game. It is not affiliated with or endorsed by the game's developer
or publisher.

## Where it stands (2026-11-25)

Tested on one machine only: Windows 11, Intel i9-12900K, 64 GB RAM, Radeon
RX 7900 XTX, 1440p, the Series X profile.

| | |
|---|---|
| Menus, videos, loading screens | 30 fps |
| Prologue, indoors | 50-75 ms a frame (13-20 fps) |
| First mission with the boat | 46-53 ms (19-22 fps) |
| Street chase with many cars and people | 100-165 ms (6-10 fps), single frames far longer |
| Outdoors on foot, the camp | 60-90 ms |
| Flying | 42-57 ms |
| Longest session so far | 41 minutes, no crash |
| Memory | 40-46 GB of private memory: **64 GB of RAM recommended** |

It started at 280 ms a frame in the same scenes two days earlier.

What is known to be wrong:

- Trees, plants and far buildings flicker outdoors in some places.
- Rooms with little direct light are too dark without `-RayTracing`.
- Textures of what just came into view appear late while the title streams.
- The first run with a new driver or a new build stutters while the driver makes
  pipelines (`victory warm` makes them ahead).
- Recording a frame in the world makes the title's own watchdog end it.

## How to run it

You need Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with shader
model 6.6 and mesh shaders, Rust (the toolchain is pinned), the MSVC build
tools, and your own dump of the title.

```powershell
git clone https://github.com/avatarmanz/VIctory-7900XTX
cd VIctory-7900XTX
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
