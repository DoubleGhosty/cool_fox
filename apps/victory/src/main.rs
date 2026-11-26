//! `victory`: the command-line front end.
//!
//! Every command reads the user's own files and writes only to stdout or to
//! the user's cache directory (outside the repository).

mod census;
mod dexnom;
mod inspect;
#[cfg(all(windows, feature = "diagnostics"))]
mod replay;
mod run;
mod shader;
mod translate;
#[cfg(windows)]
mod warm;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "victory",
    version,
    about = "Run an Xbox Series title natively on Windows"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Summarise one PE image: headers, sections, imports, TLS, build id.
    Inspect {
        file: PathBuf,
        /// List every imported function.
        #[arg(long)]
        imports: bool,
        /// List every exported function.
        #[arg(long)]
        exports: bool,
    },
    /// Aggregate the imports of every PE image under the given paths and
    /// show which ones our runtime covers. Defaults to the configured
    /// game and test-title directories.
    Census {
        paths: Vec<PathBuf>,
        /// Do not count host pass-through as covered.
        #[arg(long)]
        strict: bool,
        /// List every missing `dll!function`, most used first.
        #[arg(long)]
        missing: bool,
    },
    /// List the `dll!export` pairs an import library describes.
    LibExports {
        lib: PathBuf,
        /// List every export, not just per-DLL counts.
        #[arg(long)]
        list: bool,
    },
    /// Find shader containers in files or directories (also inside engine
    /// archives, if uncompressed) and report their layout.
    ShaderInfo {
        paths: Vec<PathBuf>,
        /// Print one line per container found.
        #[arg(long)]
        verbose: bool,
        /// Also list the decoded native instructions (implies --verbose).
        #[arg(long)]
        disasm: bool,
    },
    /// Run every shader found (containers, SGD5 files) through the
    /// translator and report what it cannot handle yet.
    Translate {
        paths: Vec<PathBuf>,
        /// Check emitted DXIL with dxil.dll's validator (`VICTORY_DXIL`,
        /// else the newest Windows SDK's).
        #[arg(long)]
        validate: bool,
        /// Also create a host D3D12 pipeline for each emitted shader.
        #[arg(long)]
        pipelines: bool,
        /// Print one line per program and outcome.
        #[arg(long)]
        list: bool,
        /// Worker threads (default: every logical core).
        #[arg(long)]
        jobs: Option<usize>,
        /// Warm the run's shader cache: a file of shader hashes (the 16
        /// hex digits a run's log names a shader by, anywhere in a line);
        /// the containers with those hashes are translated into the
        /// cache and nothing else is done. Set `VICTORY_HOST_WAVE` and
        /// `VICTORY_VIEW_LIMIT` to what the run's log says of the host.
        #[arg(long)]
        warm: Option<PathBuf>,
    },
    /// Translate every shader found (as `translate` finds them) into the
    /// shader cache, so that no run has to: a few minutes, once per change
    /// of the translator.
    Shaders {
        paths: Vec<PathBuf>,
        /// Worker threads (default: every logical core).
        #[arg(long)]
        jobs: Option<usize>,
    },
    /// Make the host pipelines of every pipeline the title ever created
    /// (runs with `VICTORY_PIPELINE_BOOK=1` write them down), so that the
    /// driver's cache on disk has them before a run needs one: once per change
    /// of the translator or the driver. Goes on where it was stopped.
    #[cfg(windows)]
    Warm {
        /// Processes at once (default: two thirds of the logical cores).
        #[arg(long)]
        jobs: Option<usize>,
        /// At most this many pipelines.
        #[arg(long)]
        limit: Option<usize>,
        /// Those made before too.
        #[arg(long)]
        again: bool,
        /// Only the pipelines earlier runs drew with.
        #[arg(long)]
        used: bool,
        /// (What the processes it starts are given.)
        #[arg(long, hide = true)]
        these: Option<String>,
    },
    /// Prepare, map and bind a title executable; with --execute, run it.
    Run {
        #[command(flatten)]
        dexnom: DexNomOptions,
        exe: PathBuf,
        /// What unimplemented imports do when called.
        #[arg(long, value_enum, default_value_t = Unimplemented::Trap)]
        unimplemented: Unimplemented,
        /// Disable host pass-through (everything must be implemented by us).
        #[arg(long)]
        strict: bool,
        /// Actually start the title. Without it, stop after binding.
        #[arg(long)]
        execute: bool,
        /// The console model the title sees (device type, memory).
        #[arg(long, value_enum, default_value_t = Console::SeriesX)]
        console: Console,
        /// Arguments for the title, after `--` (it reads the whole command
        /// line; a title that relaunches itself passes some).
        #[arg(last = true)]
        title_args: Vec<String>,
    },
    /// Run a captured frame's GPU work again (diagnostics builds; see
    /// docs/replay.md).
    #[cfg(all(windows, feature = "diagnostics"))]
    Replay(replay::Replay),
    /// Show the resolved configuration and check it.
    Config,
}

/// DexNom/VIctory's launch flags ([`dexnom`]).
#[derive(clap::Args)]
struct DexNomOptions {
    /// Ray tracing on.
    #[arg(long = "rt", conflicts_with_all = ["no_rt", "rt_static_bvh"])]
    rt: bool,
    /// Ray-structure builder, ray activation and hit resolve off.
    #[arg(long = "no-rt", conflicts_with = "rt_static_bvh")]
    no_rt: bool,
    /// Activation and resolve on, structure builder off.
    #[arg(long = "rt-static-bvh")]
    rt_static_bvh: bool,
    /// Texture levels get memory on demand.
    #[arg(long = "sparse-textures")]
    sparse_textures: bool,
    /// Textures go up on a copy queue.
    #[arg(long = "upload-ahead", conflicts_with = "no_upload_ahead")]
    upload_ahead: bool,
    /// No texture copy queue.
    #[arg(long = "no-upload-ahead")]
    no_upload_ahead: bool,
    /// Software adapter (diagnostics builds).
    #[arg(long = "warp")]
    warp: bool,
    /// Accepted for DexNom's command lines; not in this tree (ignored).
    #[arg(long = "local-gpu-memory")]
    local_gpu_memory: bool,
    /// Accepted for DexNom's command lines; not in this tree (ignored).
    #[arg(long = "native-1440p")]
    native_1440p: bool,
    /// Accepted for DexNom's command lines; not in this tree (ignored).
    #[arg(long = "1080p-to-1440p")]
    render_1080p_to_1440p: bool,
    /// Accepted for DexNom's command lines; not in this tree (ignored).
    #[arg(long = "fsr-quality")]
    fsr_quality: bool,
    /// Accepted for DexNom's command lines; not in this tree (ignored).
    #[arg(long = "fsr-balanced")]
    fsr_balanced: bool,
    /// Accepted for DexNom's command lines; not in this tree (ignored).
    #[arg(long = "fsr-performance")]
    fsr_performance: bool,
    /// Accepted for DexNom's command lines; not in this tree (ignored).
    #[arg(long = "capture-readbacks")]
    capture_readbacks: bool,
    /// Accepted for DexNom's command lines; not in this tree (ignored).
    #[arg(long = "rt-write-stats")]
    rt_write_stats: bool,
    /// Accepted for DexNom's command lines; not in this tree (ignored).
    #[arg(long = "rt-store-trace")]
    rt_store_trace: bool,
    /// Accepted for DexNom's command lines; not in this tree (ignored).
    #[arg(long = "lighting-trace")]
    lighting_trace: bool,
}

impl DexNomOptions {
    /// Sets this tree's switches. Called before the runtime starts.
    fn apply(&self) {
        let DexNomOptions { rt, no_rt, rt_static_bvh, sparse_textures, upload_ahead, no_upload_ahead, warp, local_gpu_memory, native_1440p, render_1080p_to_1440p, fsr_quality, fsr_balanced, fsr_performance, capture_readbacks, rt_write_stats, rt_store_trace, lighting_trace } = *self;
        let flags = dexnom::Flags { rt, no_rt, rt_static_bvh, sparse_textures, upload_ahead, no_upload_ahead, warp, local_gpu_memory, native_1440p, render_1080p_to_1440p, fsr_quality, fsr_balanced, fsr_performance, capture_readbacks, rt_write_stats, rt_store_trace, lighting_trace };
        let plan = dexnom::plan(flags, |key| std::env::var(key).ok());
        for (key, value) in plan.set {
            tracing::warn!(key, value, "set by a launch flag or master switch");
            // SAFETY: Windows supports setting the environment across threads;
            // a title that relaunches itself inherits it.
            unsafe { std::env::set_var(key, value) };
        }
        for name in plan.unsupported {
            tracing::warn!(name, "not in this tree: ignored");
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum Unimplemented {
    Trap,
    Zero,
}

#[derive(Clone, Copy, ValueEnum)]
enum Console {
    SeriesX,
    SeriesS,
}

fn main() -> Result<()> {
    victory_core::logging::init("info");
    // The audio recorder `run` starts for `VICTORY_AUDIO_CAPTURE`.
    #[cfg(all(windows, feature = "diagnostics"))]
    if victory_hle::modules::audio::capture::child() {
        return Ok(());
    }
    let cli = Cli::parse();
    match cli.command {
        Command::Inspect {
            file,
            imports,
            exports,
        } => inspect::run(&file, imports, exports),
        Command::Census {
            paths,
            strict,
            missing,
        } => census::run(paths, strict, missing),
        Command::LibExports { lib, list } => inspect::lib_exports(&lib, list),
        Command::ShaderInfo {
            paths,
            verbose,
            disasm,
        } => shader::run(paths, verbose || disasm, disasm),
        Command::Translate {
            paths,
            validate,
            pipelines,
            list,
            jobs,
            warm,
        } => {
            // `VICTORY_VIEW_LIMIT=<null slot>`: translate as on a host whose view heap is capped
            if let Some(n) = std::env::var("VICTORY_VIEW_LIMIT").ok().and_then(|v| v.parse().ok()) {
                victory_shader::emit::set_view_heap_limit(n);
            }
            // `VICTORY_HOST_WAVE=32`: translate as for a GPU whose waves are at most 32 lanes
            if let Some(n) = std::env::var("VICTORY_HOST_WAVE").ok().and_then(|v| v.parse().ok()) {
                victory_shader::emit::set_host_max_wave(n);
            }
            if let Some(file) = warm {
                return translate::warm(paths, &file, jobs);
            }
            translate::run(paths, validate, pipelines, list, jobs)
        }
        Command::Shaders { paths, jobs } => translate::precompile(paths, jobs),
        #[cfg(windows)]
        Command::Warm { jobs, limit, again, used, these } => warm::run(jobs, limit, again, used, these),
        Command::Run {
            dexnom,
            exe,
            unimplemented,
            strict,
            execute,
            console,
            title_args,
        } => {
            dexnom.apply();
            victory_hle::runtime::set_title_args(title_args);
            run::run(&exe, unimplemented, strict, execute, console)
        }
        #[cfg(all(windows, feature = "diagnostics"))]
        Command::Replay(options) => replay::run(options),
        Command::Config => show_config(),
    }
}

fn show_config() -> Result<()> {
    let cfg = victory_core::Config::load()?;
    let status = |p: &Option<PathBuf>| match p {
        Some(p) if p.exists() => format!("{}", p.display()),
        Some(p) => format!("{}  (does not exist)", p.display()),
        None => "(not set)".to_owned(),
    };
    println!(
        "config file : {}",
        cfg.source
            .as_ref()
            .map_or("(none)".into(), |p| p.display().to_string())
    );
    println!("game_dir    : {}", status(&cfg.game_dir));
    println!("cache_dir   : {}", cfg.cache_dir.display());
    println!("data_dir    : {}", cfg.data_dir.display());
    for t in &cfg.test_titles {
        println!("test_title  : {}", status(&Some(t.clone())));
    }
    for p in cfg
        .game_dir
        .iter()
        .chain(&cfg.test_titles)
        .chain([&cfg.cache_dir, &cfg.data_dir])
    {
        if let Err(e) = victory_core::ensure_outside_repo(p) {
            println!("POLICY VIOLATION: {e}");
        }
    }
    Ok(())
}

/// Paths given on the command line, or the configured game + test titles.
pub(crate) fn default_paths(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    if !paths.is_empty() {
        return Ok(paths);
    }
    let cfg = victory_core::Config::load()?;
    let all: Vec<PathBuf> = cfg.game_dir.into_iter().chain(cfg.test_titles).collect();
    anyhow::ensure!(
        !all.is_empty(),
        "no paths given and none configured (see victory.example.toml)"
    );
    Ok(all)
}
