use nes_core::{
    control_deck::{ControlDeck},
    input::{Player, JoypadBtn},
    mapper::{Mapper, MapperRevision},
    mem::RamState,
    ppu::Ppu,
    video::VideoFilter,
};
use image::{ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::{
    collections::hash_map::DefaultHasher,
    env,
    fs::{self, File},
    hash::{Hash, Hasher},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};
use std::io::Read;
use lazy_static::lazy_static;
use nes_core::common::{NesRegion, Regional, Reset, ResetKind};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Nes(NesState),
    Setting(Setting),
    Joypad(JoypadBtn),
}

impl From<NesState> for Action {
    fn from(state: NesState) -> Self {
        Self::Nes(state)
    }
}


impl From<Setting> for Action {
    fn from(setting: Setting) -> Self {
        Self::Setting(setting)
    }
}

impl From<JoypadBtn> for Action {
    fn from(btn: JoypadBtn) -> Self {
        Self::Joypad(btn)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum NesState {
    SoftReset,
    HardReset,
    MapperRevision(MapperRevision),
}


#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Setting {
    SetVideoFilter(VideoFilter),
    SetNesFormat(NesRegion),
}


pub(crate) const RESULT_DIR: &str = "test_results";


lazy_static! {
    static ref INIT_TESTS: bool = {
        let result_dir = PathBuf::from(RESULT_DIR);
        if result_dir.exists() {
            fs::remove_dir_all(result_dir).expect("cleared test results dir");
        }
        true
    };
    static ref PASS_DIR: PathBuf = {
        let directory = PathBuf::from(RESULT_DIR).join("pass");
        fs::create_dir_all(&directory).expect("created pass test results dir");
        directory
    };
    static ref FAIL_DIR: PathBuf = {
        let directory = PathBuf::from(RESULT_DIR).join("fail");
        fs::create_dir_all(&directory).expect("created fail test results dir");
        directory
    };
}

#[macro_export]
macro_rules! test_roms {
        ($directory:expr, $( $(#[ignore = $reason:expr])? $test:ident ),* $(,)?) => {$(
            $(#[ignore = $reason])?
            #[test]
            fn $test() {
                test_rom($directory, stringify!($test));
            }
        )*};
    }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
struct TestFrame {
    number: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hash: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slot: Option<Player>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action: Option<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
struct RomTest {
    name: String,
    frames: Vec<TestFrame>,
}

fn get_rom_tests(directory: &str) -> (PathBuf, Vec<RomTest>) {
    let file = PathBuf::from(directory)
        .join("tests")
        .with_extension("json");
    let tests = File::open(&file)
        .and_then(|file| {
            Ok(serde_json::from_reader::<_, Vec<RomTest>>(BufReader::new(
                file,
            ))?)
        })
        .expect("valid rom test data");
    (file, tests)
}

fn load_control_deck<P: AsRef<Path>>(path: P) -> ControlDeck {
    let path = path.as_ref();
    let mut rom = BufReader::new(File::open(path).expect("failed to open path"));
    let mut deck = ControlDeck::new(RamState::AllZeros);
    let mut data = vec![];
    rom.read_to_end(&mut data).unwrap();
    deck.load_rom(path.to_string_lossy().to_string(), data)
        .expect("failed to load rom");
    deck.set_filter(VideoFilter::Pixellate);
    deck.set_region(NesRegion::Ntsc);
    deck
}

fn on_frame_action(test_frame: &TestFrame, deck: &mut ControlDeck) {
    if let Some(action) = test_frame.action {
        log::debug!("{:?}", action);
        match action {
            Action::Nes(state) => match state {
                NesState::SoftReset => deck.reset(ResetKind::Soft),
                NesState::HardReset => deck.reset(ResetKind::Hard),
                NesState::MapperRevision(board) => match board {
                    MapperRevision::Mmc3(revision) => {
                        if let Mapper::Txrom(ref mut mapper) = deck.mapper_mut() {
                            mapper.set_revision(revision);
                        }
                    }
                    _ => panic!("unhandled MapperRevision {board:?}"),
                },
            },
            Action::Setting(setting) => match setting {
                Setting::SetVideoFilter(filter) => deck.set_filter(filter),
                Setting::SetNesFormat(format) => deck.set_region(format),
            },
            Action::Joypad(button) => {
                let slot = test_frame.slot.unwrap_or(Player::One);
                let joypad = deck.joypad_mut(slot);
                joypad.set_button(button.into(), true);
            }
        }
    }
}

fn on_snapshot(
    test: &str,
    test_frame: &TestFrame,
    deck: &mut ControlDeck,
    count: usize,
) -> Option<(u64, u64, u32, PathBuf)> {
    test_frame.hash.map(|expected| {
        let mut hasher = DefaultHasher::new();
        let frame_buffer = deck.frame_buffer();
        frame_buffer.hash(&mut hasher);
        let actual = hasher.finish();
        log::debug!(
                "frame : {}, matched: {}",
                test_frame.number,
                expected == actual
            );

        let result_dir = if env::var("UPDATE_SNAPSHOT").is_ok() || expected == actual {
            &*PASS_DIR
        } else {
            &*FAIL_DIR
        };
        let mut filename = test.to_owned();
        if let Some(ref name) = test_frame.name {
            let _ = write!(filename, "_{name}");
        } else if count > 0 {
            let _ = write!(filename, "_{}", count + 1);
        }
        let screenshot = result_dir
            .join(PathBuf::from(filename))
            .with_extension("png");

        ImageBuffer::<Rgba<u8>, &[u8]>::from_raw(Ppu::WIDTH, Ppu::HEIGHT, frame_buffer)
            .expect("valid frame")
            .save(&screenshot)
            .expect("result screenshot");

        (expected, actual, test_frame.number, screenshot)
    })
}

pub(crate) fn test_rom(directory: &str, test_name: &str) {
    if !&*INIT_TESTS {
        log::debug!("Initialized tests");
    }

    let (test_file, mut tests) = get_rom_tests(directory);
    let mut test = tests.iter_mut().find(|test| test.name.eq(test_name));
    assert!(test.is_some(), "No test found matching {test_name:?}");
    let test = test.as_mut().expect("definitely has a test");

    let rom = PathBuf::from(directory)
        .join(PathBuf::from(&test.name))
        .with_extension("nes");
    assert!(rom.exists(), "No test rom found for {rom:?}");

    let mut deck = load_control_deck(&rom);
    let mut results = Vec::new();
    for test_frame in test.frames.iter() {
        log::debug!("{} - {:?}", test_frame.number, deck.joypad_mut(Player::One));

        while deck.frame_number() < test_frame.number {
            deck.clock_frame().expect("valid frame clock");
            deck.clear_audio_samples();
            deck.joypad_mut(Player::One).reset(ResetKind::Soft);
            deck.joypad_mut(Player::Two).reset(ResetKind::Soft);
        }

        on_frame_action(test_frame, &mut deck);
        if let Some(result) = on_snapshot(&test.name, test_frame, &mut deck, results.len()) {
            results.push(result);
        }
    }
    let mut update_required = false;
    for (mut expected, actual, frame_number, screenshot) in results {
        if env::var("UPDATE_SNAPSHOT").is_ok() && expected != actual {
            expected = actual;
            update_required = true;
            if let Some(ref mut frame) = test
                .frames
                .iter_mut()
                .find(|frame| frame.number == frame_number)
            {
                frame.hash = Some(actual);
            }
        }
        assert_eq!(
            expected, actual,
            "mismatched snapshot for {rom:?} -> {screenshot:?}",
        );
    }
    if update_required {
        File::create(test_file)
            .and_then(|file| {
                serde_json::to_writer_pretty(BufWriter::new(file), &tests).unwrap();
                Ok(())
            })
            .expect("failed to update snapshot");
    }
}

mod cpu_tests {
    use crate::test_rom;
    test_roms!(
        "test_roms/cpu",
        branch_backward,
        nestest,
        ram_after_reset,
        regs_after_reset,
        branch_basics,
        branch_forward,
        dummy_reads,
        dummy_writes_oam,
        dummy_writes_ppumem,
        exec_space_apu,
        exec_space_ppuio,
        flag_concurrency,
        instr_abs,
        instr_abs_xy,
        instr_basics,
        instr_branches,
        instr_brk,
        instr_imm,
        instr_imp,
        instr_ind_x,
        instr_ind_y,
        instr_jmp_jsr,
        instr_misc,
        instr_rti,
        instr_rts,
        instr_special,
        instr_stack,
        instr_timing,
        instr_zp,
        instr_zp_xy,
        int_branch_delays_irq,
        int_cli_latency,
        int_irq_and_dma,
        int_nmi_and_brk,
        int_nmi_and_irq,
        overclock,
        sprdma_and_dmc_dma,
        sprdma_and_dmc_dma_512,
        timing_test,
    );
}

mod mapper_tests {
    use crate::test_rom;

    test_roms!(
        "test_roms/mapper/m004_txrom",
        a12_clocking,
        clocking,
        details,
        rev_b,
        scanline_timing,
        big_chr_ram,
        rev_a,
    );
    test_roms!("test_roms/mapper/m005_exrom", exram, basics);
}