use std::thread;
use crossbeam_channel::{bounded, Receiver, Sender};
use flutter_rust_bridge::{DartFnFuture, frb, spawn};
use nes_core::common::NesRegion;
use nes_core::control_deck::{Config, ControlDeck};
use nes_core::input::{FourPlayer, JoypadBtnState, Player};
use nes_core::mem::RamState;
use nes_core::video::VideoFilter;
use crate::api::texture::NesTexture;
use crate::fps::Fps;
use crate::frb_generated::StreamSink;

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
    ) -> NesConfig {
        NesConfig {
            filter,
            region,
            ram_state,
            four_player,
            zapper,
            genie_codes,
        }
    }
}

impl Into<Config> for NesConfig {
    fn into(self) -> Config {
        Config {
            filter: self.filter,
            region: self.region,
            ram_state: self.ram_state,
            four_player: self.four_player,
            zapper: self.zapper,
            genie_codes: self.genie_codes,
        }
    }
}

#[frb(opaque)]
pub struct NesEmulator {
    control: ControlDeck,
    tx: Sender<()>,
    rx: Receiver<()>,
}

impl NesEmulator {
    #[frb(sync)]
    pub fn create() -> NesEmulator {
        let (tx, rx) = bounded(1);
        NesEmulator {
            control: ControlDeck::new(),
            tx,
            rx,
        }
    }
    #[frb(sync)]
    pub fn with_config(config: NesConfig) -> NesEmulator {
        let (tx, rx) = bounded(1);
        NesEmulator {
            control: ControlDeck::with_config(config.into()),
            tx,
            rx,
        }
    }

    pub fn load_rom(&mut self, name: String, data: Vec<u8>) -> anyhow::Result<()> {
        self.control.load_rom(name, data)?;
        Ok(())
    }

    fn run_loop(&self, render: impl NesRender) -> anyhow::Result<()> {
        let mut control = self.control.clone();
        let rx = self.rx.clone();
        let mut fps = Fps::new(60.0);
        loop {
            if rx.try_recv().ok().is_some() {
                break;
            }
            control.clock_frame()?;
            let data = control.frame_buffer();
            render.render(data.to_vec());
            fps.tick();
        }
        Ok(())
    }
    pub async fn run_loop_for_callback(&self, callback: impl Fn(Vec<u8>) -> DartFnFuture<()>) -> anyhow::Result<()> {
        let mut control = self.control.clone();
        let rx = self.rx.clone();
        let mut fps = Fps::new(60.0);
        loop {
            if rx.try_recv().ok().is_some() {
                break;
            }
            control.clock_frame()?;
            let data = control.frame_buffer();
            callback(data.to_vec()).await;
            fps.tick();
        }
        Ok(())
    }
    pub fn run_loop_for_painter(&self, sink: StreamSink<Vec<u8>>) -> anyhow::Result<()> {
        self.run_loop(sink)?;
        Ok(())
    }
    pub fn run_loop_for_texture(&self, texture: NesTexture) -> anyhow::Result<()> {
        self.run_loop(texture)?;
        Ok(())
    }

    pub fn handle_button(&mut self, player: Player, button: NesButton, pressed: bool) {
        let joypad = &mut self.control.joypad_mut(player.into());
        joypad.set_button(button.into(), pressed);
    }

    #[frb(sync)]
    pub fn stop_loop(&self) {
        let tx = self.tx.clone();
        let _ = tx.send(());
    }
}

trait NesRender: Send + 'static {
    fn render(&self, data: Vec<u8>);
}

impl NesRender for StreamSink<Vec<u8>> {
    fn render(&self, data: Vec<u8>) {
        let _ = self.add(data);
    }
}

impl NesRender for NesTexture {
    fn render(&self, data: Vec<u8>) {
        let _ = self.render(data);
    }
}