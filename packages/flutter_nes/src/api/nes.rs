use anyhow::anyhow;
use flutter_rust_bridge::{DartFnFuture, frb};
use nes_core::common::NesRegion;
use nes_core::control_deck::{ControlDeck};
use nes_core::genie::GenieCode;
use nes_core::input::{FourPlayer, JoypadBtnState, Player};
use nes_core::mem::RamState;
use nes_core::video::VideoFilter;
use crate::api::texture::NesTexture;
use crate::fps::Fps;

impl Into<JoypadBtnState> for NesButton {
    fn into(self) -> JoypadBtnState {
        match self {
            NesButton::Start => JoypadBtnState::START,
            NesButton::Select => JoypadBtnState::SELECT,
            NesButton::TurboA => JoypadBtnState::TURBO_A,
            NesButton::TurboB => JoypadBtnState::TURBO_B,
            NesButton::A => JoypadBtnState::A,
            NesButton::B => JoypadBtnState::B,
            NesButton::Up => JoypadBtnState::UP,
            NesButton::Down => JoypadBtnState::DOWN,
            NesButton::Left => JoypadBtnState::LEFT,
            NesButton::Right => JoypadBtnState::RIGHT,
        }
    }
}

pub enum NesButton {
    Start,
    Select,
    TurboA,
    TurboB,
    A,
    B,
    Up,
    Down,
    Left,
    Right,
}

pub struct NesConfig {
    pub filter: VideoFilter,
    pub region: NesRegion,
    pub ram_state: RamState,
    pub four_player: FourPlayer,
    pub zapper: bool,
    pub genie_codes: Vec<String>,
    pub fps: u32,
}

impl NesConfig {
    #[frb(sync)]
    pub fn create(
        filter: VideoFilter,
        region: NesRegion,
        ram_state: RamState,
        four_player: FourPlayer,
        zapper: bool,
        genie_codes: Vec<String>,
        fps: u32,
    ) -> NesConfig {
        NesConfig {
            filter,
            region,
            ram_state,
            four_player,
            zapper,
            genie_codes,
            fps,
        }
    }
}

// impl Into<Config> for NesConfig {
//     fn into(self) -> Config {
//         let mut codes = Vec::new();
//         for genie_code in self.genie_codes {
//             if let Ok(code) = GenieCode::new(genie_code) {
//                 codes.push(code)
//             }
//         }
//         Config {
//             filter: self.filter,
//             region: self.region,
//             ram_state: self.ram_state,
//             four_player: self.four_player,
//             zapper: self.zapper,
//             genie_codes: codes,
//         }
//     }
// }

#[frb(opaque)]
pub struct NesEmulator {
    control: ControlDeck,
    fps: Fps,
}

impl NesEmulator {
    #[frb(sync)]
    pub fn create() -> NesEmulator {
        NesEmulator {
            control: ControlDeck::new(RamState::AllZeros),
            fps: Fps::new(120),
        }
    }
    #[frb(sync)]
    pub fn with_config(config: NesConfig) -> NesEmulator {
        let fps = config.fps;
        NesEmulator {
            control: ControlDeck::new(RamState::AllZeros),
            fps: Fps::new(fps),
        }
    }

    pub fn load_rom(&mut self, name: String, data: Vec<u8>) -> anyhow::Result<()> {
        self.control.load_rom(name, data)?;
        Ok(())
    }

    pub async fn run_loop(&mut self, on_data: impl Fn(Vec<u8>) -> DartFnFuture<()>) -> anyhow::Result<()> {
        loop {
            self.control.clock_frame()?;
            let data = self.control.frame_buffer();
            on_data(data.to_vec()).await;
            self.fps.tick();
        }
    }
    pub fn run_loop_for_texture(&mut self, texture: NesTexture) -> anyhow::Result<()> {
        loop {
            self.control.clock_frame()?;
            let data = self.control.frame_buffer();
            texture.render(data.to_vec())?;
            self.fps.tick();
        }
    }

    pub fn handle_button(&mut self, player: Player, button: NesButton, pressed: bool) {
        let joypad = &mut self.control.joypad_mut(player.into());
        joypad.set_button(button.into(), pressed);
    }
}
