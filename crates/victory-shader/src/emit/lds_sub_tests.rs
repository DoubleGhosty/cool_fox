//! Group-shared atomic subtracts on the host (#129): `ds_sub_rtn_u32`, and
//! `ds_add_rtn_u32` of a negated value, make pipelines NVIDIA's compiler takes
//! and give each lane the console's old value.

use super::host::{Dispatch, Host, Root, View};
use super::{HostShader, lower, validator};
use crate::native::{
    ComputeIo, HwStage, NativeProgram, ResourceType, RootParam, RootType, StageIo, UsedResource,
};
use crate::{ShaderError, io, ir, resource, structure};

const THREADS: u32 = 32;
const START: u32 = 1000;

/// One wave of 32 lanes: LDS dword 0 starts at [`START`]; lane t takes t + 1
/// from it (with `ds_sub_rtn_u32`, or with `ds_add_rtn_u32` of `0 - (t + 1)`
/// when `negated_add`); out[t] is the old value it got, out[32 + t] the dword
/// after every lane's.
fn native(negated_add: bool) -> NativeProgram {
    let mov = |dst: u32, src: u32| 0x7E00_0200 | dst << 17 | src;
    let smov = |dst: u32, src: u32| 0xBE80_0300 | dst << 16 | src;
    let ds = |op: u32, vdst: u32, data: u32, addr: u32| [0x36 << 26 | op << 18, vdst << 24 | data << 8 | addr];
    let store = |vdata: u32, offset: u32| [0x38 << 26 | 0x1C << 18 | 1 << 12 | offset, 0x80 << 24 | 2 << 16 | vdata << 8 | 7];
    let mut words = vec![
        smov(5, 128),
        0x3D << 26 | 0x02 << 18 | 8 << 6 | 2,
        125 << 25,   // s_load_dwordx4 s[8:11], s[4:5], 0
        0x340E_0082, // v_lshlrev_b32 v7, 2, v0 (output byte offset)
        mov(1, 128), // v_mov_b32 v1, 0 (the LDS address)
        0x7E04_02FF,
        START, // v_mov_b32 v2, START
    ];
    words.extend(ds(0x0D, 0, 2, 1)); // ds_write_b32 v1, v2
    words.push(0xBF8A_0000); // s_barrier
    words.push(0x4A06_0081); // v_add_nc_u32 v3, 1, v0
    if negated_add {
        words.push(0x4C0A_0680); // v_sub_nc_u32 v5, 0, v3
        words.extend(ds(0x20, 4, 5, 1)); // ds_add_rtn_u32 v4, v1, v5
    } else {
        words.extend(ds(0x21, 4, 3, 1)); // ds_sub_rtn_u32 v4, v1, v3
    }
    words.push(0xBF8A_0000); // s_barrier
    words.extend(ds(0x36, 6, 0, 1)); // ds_read_b32 v6, v1
    words.extend(store(4, 0)); // buffer_store_dword v4, v7, s[8:11], 0 offen
    words.extend(store(6, 4 * THREADS)); // ... v6 at offset 128
    words.push(0xBF81_0000); // s_endpgm
    NativeProgram {
        stage: HwStage::Cs,
        wave64: false,
        vgprs: 8,
        layout: vec![
            RootParam {
                root: RootType::Constants,
                table_offset_dwords: 0,
                sgprs: 4,
                base_register: 0,
                user_data: 0,
                slot: 0,
            },
            RootParam {
                root: RootType::Table,
                table_offset_dwords: 0,
                sgprs: 1,
                base_register: 0,
                user_data: 4,
                slot: 1,
            },
        ],
        used: vec![UsedResource {
            kind: ResourceType::Uav,
            root: RootType::Table,
            table_offset_dwords: 0,
            lower_bound: 0,
            stride: 0,
            dwords: 4,
            user_data: 4,
            slot: 1,
        }],
        io: StageIo::Cs(ComputeIo {
            // LDS: one granule; five user SGPRs.
            rsrc2: 1 << 15 | 5 << 1,
            threads: [THREADS, 1, 1],
        }),
        code: words.iter().flat_map(|w| w.to_le_bytes()).collect(),
    }
}

fn lowered(native: &NativeProgram) -> Result<HostShader, ShaderError> {
    let p = ir::build(&native.code, native.wave64, native.vgprs).expect("ir");
    let st = structure::structure(&p).expect("structure");
    let res = resource::track(&p, &native.layout, native.stage, Some(&st)).expect("resources");
    let io = io::raise(&p, native).expect("io");
    let was = lower::TEST_HOST_WAVE.replace(Some(32));
    let result = lower::lower(&lower::Parts {
        native,
        p: &p,
        st: &st,
        io: &io,
        res: &res,
        inputs: Vec::new(),
        outputs: Vec::new(),
        prims: Vec::new(),
        mesh_stride: 0,
    });
    lower::TEST_HOST_WAVE.set(was);
    let result = result?;
    let shader = HostShader {
        stage: HwStage::Cs,
        dxil: result.dxil,
        user_data: result.user_data,
        samplers: result.samplers,
        static_samplers: Vec::new(),
        root_cbvs: result.root_cbvs,
        root_srvs: result.root_srvs,
        root_uavs: result.root_uavs,
        root_indexed_views: res.root_indexed_views,
    };
    match validator::Validator::load(validator::Validator::default_path().as_deref()) {
        Ok(mut v) => v.validate(&mut shader.dxil.clone()).expect("DXIL validation"),
        Err(e) => eprintln!("DXIL validator unavailable: {e}"),
    }
    Ok(shader)
}

#[test]
fn lds_subtracts_make_pipelines_and_give_the_console_old_values() {
    let _one = super::valu_tests::ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let Ok(host) = Host::new(false) else {
        eprintln!("no SM 6.6 hardware; skipped");
        return;
    };
    let mut options = windows::Win32::Graphics::Direct3D12::D3D12_FEATURE_DATA_D3D12_OPTIONS1::default();
    // SAFETY: options is the correctly sized output for this feature query.
    unsafe {
        host.device
            .CheckFeatureSupport(
                windows::Win32::Graphics::Direct3D12::D3D12_FEATURE_D3D12_OPTIONS1,
                (&raw mut options).cast(),
                size_of_val(&options) as u32,
            )
            .expect("wave capabilities");
    }
    if options.WaveLaneCountMin > 32 || options.WaveLaneCountMax < 32 {
        eprintln!("no 32-lane waves; skipped");
        return;
    }
    for negated_add in [false, true] {
        let shader = lowered(&native(negated_add)).expect("lower");
        let (root, pso) = host
            .compute_pipeline(&shader)
            .unwrap_or_else(|e| panic!("pipeline (negated add {negated_add}): {e}"));
        let ud = vec![0; shader.user_data_dwords() as usize];
        let out = host
            .dispatch(
                &root,
                &pso,
                &Dispatch {
                    roots: vec![(0, Root::Constants(ud))],
                    views: &[View::RawUav(2 * THREADS)],
                    samplers: &[],
                    groups: [1, 1, 1],
                },
            )
            .expect("dispatch");
        let out = &out[0];
        let total: u32 = (1..=THREADS).sum();
        for t in 0..THREADS as usize {
            assert_eq!(out[THREADS as usize + t], START - total, "lane {t}: the dword after");
        }
        // Whatever order the lanes took, their old values chain: from START,
        // each the one before less what that lane took.
        let mut olds: Vec<(u32, u32)> = (0..THREADS).map(|t| (out[t as usize], t + 1)).collect();
        olds.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        let mut expect = START;
        for (old, took) in olds {
            assert_eq!(old, expect, "negated add {negated_add}");
            expect -= took;
        }
    }
}
