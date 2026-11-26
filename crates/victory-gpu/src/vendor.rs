//! Which Windows GPU the process runs on, and the budgets that follow.
//!
//! The RTX residency profile was measured on a 32 GB card (about 18 GB in
//! use). An RX 9070 XT has 16 GB. These functions pick the discrete AMD
//! adapter and scale idle reserves to that card's WDDM budget. No Direct3D
//! calls live here, so the rules can be tested without a device.

/// PCI vendor id for AMD / Radeon, including the RX 9070 XT.
pub const AMD: u32 = 0x1002;
/// PCI vendor id for NVIDIA.
pub const NVIDIA: u32 = 0x10DE;

const GIB: u64 = 1 << 30;

/// What `VICTORY_GPU` and `VICTORY_D3D12_ADAPTER` ask for. Both apply.
/// An empty request accepts every hardware adapter, and the caller prefers
/// the one with the most dedicated memory (the 9070 XT over a Ryzen iGPU).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuFilter {
    pub vendor: Option<u32>,
    pub index: Option<u32>,
    pub name: Option<String>,
}

/// One DXGI adapter, minus the COM object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterFact {
    pub index: u32,
    pub vendor: u32,
    pub device: u32,
    pub dedicated: u64,
    pub software: bool,
    pub name: String,
}

/// How hard texture prefetch should be, from the card's local budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VramPace {
    /// No sample yet.
    Unknown,
    Open,
    Tight,
    Critical,
}

/// `VICTORY_GPU` is `amd` / `radeon` / `nvidia` / `any` (or a name fragment).
/// `VICTORY_D3D12_ADAPTER` is a DXGI index or a fragment of the adapter name
/// (`9070` matches "AMD Radeon RX 9070 XT").
pub fn gpu_filter(gpu: Option<&str>, adapter: Option<&str>) -> GpuFilter {
    let mut filter = GpuFilter { vendor: None, index: None, name: None };
    if let Some(gpu) = word(gpu) {
        match gpu.as_str() {
            "amd" | "radeon" | "rdna" => filter.vendor = Some(AMD),
            "nvidia" | "geforce" | "rtx" => filter.vendor = Some(NVIDIA),
            "any" => {}
            other => filter.name = Some(other.to_owned()),
        }
    }
    if let Some(adapter) = word(adapter) {
        // DXGI indexes are 0, 1, 2. A product number such as 9070 is a name.
        if let Ok(index) = adapter.parse::<u32>()
            && index < 32
        {
            filter.index = Some(index);
        } else {
            filter.name = Some(adapter);
        }
    }
    filter
}

fn word(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|s| !s.is_empty()).map(|s| s.to_ascii_lowercase())
}

/// `false` for a software adapter or one the filter excludes.
pub fn adapter_matches(filter: &GpuFilter, fact: &AdapterFact) -> bool {
    if fact.software {
        return false;
    }
    if let Some(vendor) = filter.vendor
        && fact.vendor != vendor
    {
        return false;
    }
    if let Some(index) = filter.index
        && fact.index != index
    {
        return false;
    }
    if let Some(name) = &filter.name
        && !fact.name.to_ascii_lowercase().contains(name)
    {
        return false;
    }
    true
}

/// Eligible adapters, most dedicated memory first. Equal memory keeps the
/// lower DXGI index, which is the stable choice.
pub fn rank_adapters<'a>(filter: &GpuFilter, facts: &'a [AdapterFact]) -> Vec<&'a AdapterFact> {
    let mut ranked: Vec<&AdapterFact> = facts.iter().filter(|fact| adapter_matches(filter, fact)).collect();
    ranked.sort_by(|a, b| b.dedicated.cmp(&a.dedicated).then(a.index.cmp(&b.index)));
    ranked
}

/// Navi 48. Public PCI ids: 0x7550 is the RX 9070 XT, 0x7551 the RX 9070.
const RDNA4_DEVICES: &[u32] = &[0x7550, 0x7551];

/// RDNA 4, the RX 9000 cards. The shader compiler in Adrenalin
/// (`amdxc64.dll`) faults on DXIL that RDNA 3 accepts, so the translator
/// takes a different path for these ids and for a name that says 9070,
/// 9060 or 9050.
pub fn rdna4(vendor: u32, device: u32, name: &str) -> bool {
    if vendor != AMD {
        return false;
    }
    if RDNA4_DEVICES.contains(&device) {
        return true;
    }
    let name = name.to_ascii_lowercase();
    name.contains("9070") || name.contains("9060") || name.contains("9050") || name.contains("rdna 4") || name.contains("rdna4")
}

/// Label stored in the log. Dedicated bytes are DXGI's figure, which for an
/// RX 9070 XT is 16 GB (a little under that after the OS reserve is not
/// applied here; the budget check uses the WDDM budget separately).
pub fn profile_label(vendor: u32, dedicated: u64) -> &'static str {
    if vendor == AMD && dedicated > 12 * GIB && dedicated < 20 * GIB {
        "RDNA 16 GB (RX 9070 XT class)"
    } else if vendor == AMD && dedicated >= 20 * GIB {
        "RDNA 24 GB+"
    } else if vendor == AMD {
        "RDNA"
    } else if vendor == NVIDIA {
        "NVIDIA"
    } else {
        "other"
    }
}

/// How much system RAM the big pools may hold. A 32 GB kit reports a little
/// under 32 GiB. The old pools were sized on a 64 GB machine, where a run
/// sat at 40-46 GB private. That does not fit in 32 GB next to Windows and
/// the driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostRam {
    /// Host lists kept in the air before a flight is settled.
    pub late_lists: usize,
    /// Host lists reused after they have run.
    pub idle_lists: usize,
    /// Upload bytes those idle lists may keep.
    pub idle_upload: u64,
    /// Title memory mirrored for shaders that read by address, in MiB.
    pub memory_card_mb: u64,
    /// Pages of title memory held for address reads (64 KiB each).
    pub memory_pages: usize,
    /// Submissions an object waits before it is dropped.
    pub bury_lag: u64,
    /// Pipeline compiler threads.
    pub pipeline_workers: usize,
    /// Cold texture staging may sit above this, then the least recently
    /// used of it goes. Zero: only the idle timeout applies.
    pub staging_ceiling: u64,
    /// Snapshot buffers kept so the submitting thread does not allocate
    /// them again.
    pub snapshot_bytes: usize,
    /// How soon the pipeline library's mapped file may leave the working
    /// set. Zero: the ordinary idle timeout.
    pub library_quiet_ms: u64,
}

/// `total_phys` is what Windows reports (`ullTotalPhys`).
pub fn host_ram(total_phys: u64) -> HostRam {
    if total_phys > 40 * GIB {
        HostRam {
            late_lists: 256,
            idle_lists: 512,
            idle_upload: 256 << 20,
            memory_card_mb: 4096,
            memory_pages: 3 << 14,
            bury_lag: 64,
            pipeline_workers: 16,
            staging_ceiling: 0,
            snapshot_bytes: 512 << 20,
            library_quiet_ms: 0,
        }
    } else if total_phys > 20 * GIB {
        // 32 GB. Lists in the air stay near the measured count: 128 still
        // stopped the submitting thread, 256 did not. 192 is the cut that
        // keeps that path. What is not in a frame is released elsewhere.
        HostRam {
            late_lists: 192,
            idle_lists: 160,
            idle_upload: 128 << 20,
            memory_card_mb: 1536,
            memory_pages: 1 << 14,
            bury_lag: 8,
            pipeline_workers: 4,
            staging_ceiling: 1536 << 20,
            snapshot_bytes: 384 << 20,
            library_quiet_ms: 2_000,
        }
    } else {
        HostRam {
            late_lists: 128,
            idle_lists: 128,
            idle_upload: 96 << 20,
            memory_card_mb: 768,
            memory_pages: 1 << 13,
            bury_lag: 8,
            pipeline_workers: 2,
            staging_ceiling: 768 << 20,
            snapshot_bytes: 256 << 20,
            library_quiet_ms: 2_000,
        }
    }
}

/// Share of the local budget the texture queue may fill.
///
/// `override_percent` is `VICTORY_TEXTURE_ROOM_PERCENT` when it parses.
/// Otherwise a 16 GB card (WDDM budget under 20 GiB, which is an RX 9070 XT)
/// stops at 82 so the swap chain, the driver's compiler and the reserved
/// render targets still fit. 20 GiB and above keep the measured 90.
pub fn texture_room_percent(budget: Option<u64>, override_percent: Option<u64>) -> u64 {
    if let Some(percent) = override_percent {
        return percent.clamp(50, 97);
    }
    match budget {
        Some(budget) if budget < 20 * GIB => 82,
        _ => 90,
    }
}

/// Empty 64 MiB texture blocks kept while the card is under 85% of budget.
///
/// 24 GB and above keep the previous 4 GiB (64 blocks). A 16 GB 9070 XT
/// keeps 1 GiB. Smaller cards keep 256 MiB. Over the line, or with no
/// budget, only two blocks stay.
pub fn spare_blocks(budget: u64, used: u64, keep: bool) -> usize {
    const SPARE: usize = 2;
    if !keep || budget == 0 || u128::from(used) * 100 >= u128::from(budget) * 85 {
        return SPARE;
    }
    if budget >= 20 * GIB {
        64
    } else if budget > 12 * GIB {
        16
    } else {
        4
    }
}

/// Prefetch pace from the local budget and what this process already holds.
pub fn vram_pace(budget: u64, used: u64) -> VramPace {
    if budget == 0 {
        return VramPace::Unknown;
    }
    let head = budget.saturating_sub(used);
    let (critical, tight, full) = if budget < 20 * GIB {
        // 16 GB class: stop speculative uploads earlier than on a 24 GB card.
        (1 * GIB, 2560 << 20, 88)
    } else {
        (1536 << 20, 3 * GIB, 92)
    };
    if head < critical || used.saturating_mul(100) > budget.saturating_mul(full) {
        VramPace::Critical
    } else if head < tight || used.saturating_mul(100) > budget.saturating_mul(full - 8) {
        VramPace::Tight
    } else {
        VramPace::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(index: u32, vendor: u32, dedicated_gib: u64, name: &str) -> AdapterFact {
        AdapterFact {
            index,
            vendor,
            device: 0,
            dedicated: dedicated_gib * GIB,
            software: false,
            name: name.to_owned(),
        }
    }

    #[test]
    fn a_9070_xt_beats_the_igpu_and_ignores_nvidia() {
        let facts = vec![
            card(0, AMD, 1, "AMD Radeon Graphics"),
            card(1, NVIDIA, 32, "NVIDIA GeForce RTX 5090"),
            card(2, AMD, 16, "AMD Radeon RX 9070 XT"),
            AdapterFact {
                index: 3,
                vendor: 0x1414,
                device: 0x8c,
                dedicated: 0,
                software: true,
                name: "Microsoft Basic Render Driver".into(),
            },
        ];
        let filter = gpu_filter(Some("amd"), None);
        let ranked = rank_adapters(&filter, &facts);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].name, "AMD Radeon RX 9070 XT");
        assert_eq!(ranked[1].index, 0);
        assert_eq!(profile_label(AMD, 16 * GIB), "RDNA 16 GB (RX 9070 XT class)");
        assert!(rdna4(AMD, 0x7550, "AMD Radeon RX 9070 XT"));
        assert!(rdna4(AMD, 0, "AMD Radeon RX 9070"));
        assert!(!rdna4(AMD, 0x744c, "AMD Radeon RX 7900 XTX"));
        assert!(!rdna4(NVIDIA, 0x7550, "NVIDIA GeForce RTX 5090"));
        let ram = host_ram(32 * GIB - GIB);
        assert_eq!(ram.late_lists, 192);
        assert_eq!(ram.memory_card_mb, 1536);
        assert_eq!(ram.pipeline_workers, 4);
        assert_eq!(ram.staging_ceiling, 1536 << 20);
        assert_eq!(host_ram(64 * GIB).late_lists, 256);
        assert_eq!(host_ram(64 * GIB).staging_ceiling, 0);
        assert_eq!(host_ram(16 * GIB).pipeline_workers, 2);
    }

    #[test]
    fn the_name_9070_selects_that_card_only() {
        let facts = vec![
            card(0, AMD, 24, "AMD Radeon RX 7900 XTX"),
            card(1, AMD, 16, "AMD Radeon RX 9070 XT"),
        ];
        let filter = gpu_filter(Some("amd"), Some("9070"));
        let ranked = rank_adapters(&filter, &facts);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].index, 1);
        assert!(ranked[0].name.contains("9070"));
    }

    #[test]
    fn an_index_is_not_parsed_as_a_name() {
        let filter = gpu_filter(Some("nvidia"), Some("1"));
        assert_eq!(filter.vendor, Some(NVIDIA));
        assert_eq!(filter.index, Some(1));
        assert_eq!(filter.name, None);
    }

    #[test]
    fn sixteen_gigabytes_keeps_a_smaller_reserve_and_an_earlier_ceiling() {
        let budget = 15 * GIB;
        assert_eq!(texture_room_percent(Some(budget), None), 82);
        assert_eq!(texture_room_percent(Some(22 * GIB), None), 90);
        assert_eq!(texture_room_percent(Some(budget), Some(90)), 90);
        assert_eq!(spare_blocks(budget, GIB, true), 16);
        assert_eq!(spare_blocks(22 * GIB, GIB, true), 64);
        assert_eq!(spare_blocks(8 * GIB, GIB, true), 4);
        assert_eq!(spare_blocks(budget, budget / 100 * 90, true), 2);
        assert_eq!(vram_pace(0, 0), VramPace::Unknown);
        assert_eq!(vram_pace(budget, 4 * GIB), VramPace::Open);
        assert_eq!(vram_pace(budget, budget - (2 * GIB)), VramPace::Tight);
        assert_eq!(vram_pace(budget, budget - (512 << 20)), VramPace::Critical);
    }
}
