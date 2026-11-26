//! DexNom/VIctory's launch flags and master switches, mapped onto this tree's
//! own `VICTORY_...` switches. Only what has a counterpart here changes
//! anything; the rest is named in a warning and otherwise ignored.

/// The flags as given on the command line.
#[derive(Default, Clone, Copy)]
pub struct Flags {
    pub rt: bool,
    pub no_rt: bool,
    pub rt_static_bvh: bool,
    pub sparse_textures: bool,
    pub upload_ahead: bool,
    pub no_upload_ahead: bool,
    pub warp: bool,
    pub local_gpu_memory: bool,
    pub native_1440p: bool,
    pub render_1080p_to_1440p: bool,
    pub fsr_quality: bool,
    pub fsr_balanced: bool,
    pub fsr_performance: bool,
    pub capture_readbacks: bool,
    pub rt_write_stats: bool,
    pub rt_store_trace: bool,
    pub lighting_trace: bool,
}

/// What to set, and what was asked for that this tree does not have.
#[derive(Default, Debug)]
pub struct Plan {
    pub set: Vec<(&'static str, &'static str)>,
    pub unsupported: Vec<&'static str>,
}

/// Works out the plan from the flags and the environment (`get`).
pub fn plan(f: Flags, get: impl Fn(&str) -> Option<String>) -> Plan {
    let mut p = Plan::default();
    // A flag wins over `VICTORY_RAY_TRACING`; without either, the separate
    // switches (`VICTORY_BVH_BUILDER` and the others) stay as they are.
    let mode = if f.rt_static_bvh {
        Some("static-bvh")
    } else if f.no_rt {
        Some("0")
    } else if f.rt {
        Some("1")
    } else {
        match get("VICTORY_RAY_TRACING").as_deref() {
            Some("0") => Some("0"),
            Some("1") => Some("1"),
            Some("static-bvh") => Some("static-bvh"),
            _ => None,
        }
    };
    if let Some(mode) = mode {
        let (builder, passes) = match mode {
            "1" => ("1", "1"),
            "0" => ("0", "0"),
            _ => ("0", "1"),
        };
        p.set.extend([
            ("VICTORY_RAY_TRACING", mode),
            ("VICTORY_BVH_BUILDER", builder),
            ("VICTORY_ACTIVATE_RAYS", passes),
            ("VICTORY_RT_RESOLVE", passes),
        ]);
    }
    // The stand-in light is this tree's own (`VICTORY_AMBIENT`), not
    // DexNom's material-shader fill. It must not lie over traced light.
    match (mode, get("VICTORY_AMBIENT_FALLBACK").as_deref()) {
        (Some("static-bvh"), _) | (_, Some("0")) => p.set.push(("VICTORY_AMBIENT", "0")),
        (_, Some("1")) if get("VICTORY_AMBIENT").is_none() => {
            p.set.push(("VICTORY_AMBIENT", "constant:0.8"));
        }
        _ => {}
    }
    if f.sparse_textures {
        p.set.push(("VICTORY_LEVELS_ON_DEMAND", "1"));
    }
    if f.upload_ahead {
        p.set.push(("VICTORY_UPLOAD_AHEAD", "1"));
    }
    if f.no_upload_ahead {
        p.set.push(("VICTORY_UPLOAD_AHEAD", "0"));
    }
    if f.warp {
        p.set.push(("VICTORY_D3D12_WARP", "1"));
    }
    let env_on = |key: &str| get(key).is_some_and(|v| v != "0");
    p.unsupported = [
        (f.local_gpu_memory, "--local-gpu-memory"),
        (f.native_1440p, "--native-1440p"),
        (f.render_1080p_to_1440p, "--1080p-to-1440p"),
        (f.fsr_quality, "--fsr-quality"),
        (f.fsr_balanced, "--fsr-balanced"),
        (f.fsr_performance, "--fsr-performance"),
        (f.capture_readbacks, "--capture-readbacks"),
        (f.rt_write_stats, "--rt-write-stats"),
        (f.rt_store_trace, "--rt-store-trace"),
        (f.lighting_trace, "--lighting-trace"),
        (env_on("VICTORY_FSR_4K"), "VICTORY_FSR_4K"),
        (
            f.warp && !cfg!(feature = "diagnostics"),
            "--warp (needs a diagnostics build)",
        ),
    ]
    .into_iter()
    .filter_map(|(asked, name)| asked.then_some(name))
    .collect();
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(p: &Plan, key: &str) -> Option<&'static str> {
        p.set.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }

    #[test]
    fn nothing_asked_changes_nothing() {
        let p = plan(Flags::default(), |_| None);
        assert!(p.set.is_empty() && p.unsupported.is_empty());
    }

    #[test]
    fn flag_wins_over_the_environment() {
        let env = |k: &str| (k == "VICTORY_RAY_TRACING").then(|| "0".to_owned());
        let p = plan(
            Flags {
                rt: true,
                ..Flags::default()
            },
            env,
        );
        assert_eq!(value(&p, "VICTORY_BVH_BUILDER"), Some("1"));
        assert_eq!(
            value(&plan(Flags::default(), env), "VICTORY_BVH_BUILDER"),
            Some("0")
        );
    }

    #[test]
    fn static_bvh_keeps_the_passes_without_the_builder() {
        let p = plan(
            Flags {
                rt_static_bvh: true,
                ..Flags::default()
            },
            |_| None,
        );
        assert_eq!(value(&p, "VICTORY_BVH_BUILDER"), Some("0"));
        assert_eq!(value(&p, "VICTORY_RT_RESOLVE"), Some("1"));
        assert_eq!(value(&p, "VICTORY_AMBIENT"), Some("0"));
    }

    #[test]
    fn missing_features_are_named() {
        let f = Flags {
            fsr_quality: true,
            rt_write_stats: true,
            sparse_textures: true,
            ..Flags::default()
        };
        let p = plan(f, |_| None);
        assert_eq!(p.unsupported, ["--fsr-quality", "--rt-write-stats"]);
        assert_eq!(value(&p, "VICTORY_LEVELS_ON_DEMAND"), Some("1"));
    }
}
