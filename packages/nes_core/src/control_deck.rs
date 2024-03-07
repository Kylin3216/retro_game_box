use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::ops::ControlFlow;
use crate::{
    apu::{Apu, Channel},
    bus::Bus,
    cart::Cart,
    common::{Clock, ResetKind, NesRegion, Regional, Reset},
    cpu::Cpu,
    input::{FourPlayer, Joypad, Player},
    mapper::Mapper,
    mem::RamState,
    ppu::Ppu,
    video::{Video, VideoFilter},
};
use anyhow::{anyhow, Result};
use crate::genie::GenieCode;

/// Represents an NES Control Deck
#[derive(Debug, Clone)]
#[must_use]
pub struct ControlDeck {
    running: bool,
    video: Video,
    loaded_rom: Option<String>,
    cycles_remaining: f32,
    cpu: Cpu,
}

impl Default for ControlDeck {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Config {
    pub filter: VideoFilter,
    pub region: NesRegion,
    pub ram_state: RamState,
    pub four_player: FourPlayer,
    pub zapper: bool,
    pub genie_codes: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            filter:
            VideoFilter::default(),
            region: NesRegion::default(),
            ram_state: RamState::AllZeros,
            four_player: FourPlayer::default(),
            zapper: false,
            genie_codes: vec![],
        }
    }
}

impl ControlDeck {
    /// Create a NES `ControlDeck`.
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }
    pub fn with_config(config: Config) -> Self {
        let mut cpu = Cpu::new(Bus::new(config.ram_state));
        cpu.set_region(config.region);
        cpu.set_four_player(config.four_player);
        cpu.connect_zapper(config.zapper);
        for genie_code in config.genie_codes.iter().cloned() {
            let _ = cpu.add_genie_code(genie_code);
        }
        let mut video = Video::default();
        video.set_filter(config.filter);
        Self {
            running: false,
            video,
            loaded_rom: None,
            cycles_remaining: 0.0,
            cpu,
        }
    }

    /// Loads a ROM cartridge into memory
    ///
    /// # Errors
    ///
    /// If there is any issue loading the ROM, then an error is returned.
    pub fn load_rom(&mut self, name: String, rom: Vec<u8>) -> Result<()> {
        self.loaded_rom = Some(name.clone());
        let cart = Cart::from_rom(name, rom, self.cpu.ram_state())?;
        self.set_region(cart.region());
        self.cpu.load_cart(cart);
        self.reset(ResetKind::Hard);
        Ok(())
    }

    #[inline]
    pub fn load_cpu(&mut self, cpu: Cpu) {
        self.cpu = cpu;
    }

    #[inline]
    #[must_use]
    pub const fn loaded_rom(&self) -> &Option<String> {
        &self.loaded_rom
    }

    #[inline]
    #[must_use]
    pub const fn cart_battery_backed(&self) -> bool {
        self.cpu.cart_battery_backed()
    }

    #[inline]
    #[must_use]
    pub fn sram(&self) -> &[u8] {
        self.cpu.sram()
    }

    #[inline]
    pub fn load_sram(&mut self, sram: Vec<u8>) {
        self.cpu.load_sram(sram);
    }

    #[inline]
    #[must_use]
    pub fn wram(&self) -> &[u8] {
        self.cpu.wram()
    }

    /// Get a frame worth of pixels.
    #[inline]
    #[must_use]
    pub fn frame_buffer(&mut self) -> &[u8] {
        self.video
            .apply_filter(self.cpu.frame_buffer(), self.cpu.frame_number());
        self.video.output()
    }

    /// Get the current frame number.
    #[inline]
    #[must_use]
    pub const fn frame_number(&self) -> u32 {
        self.cpu.frame_number()
    }

    /// Audio sample rate.
    #[inline]
    #[must_use]
    pub const fn sample_rate(&self) -> f32 {
        self.cpu.clock_rate()
    }

    /// Get audio samples.
    #[inline]
    #[must_use]
    pub fn audio_samples(&self) -> &[f32] {
        self.cpu.audio_samples()
    }

    /// Clear audio samples.
    #[inline]
    pub fn clear_audio_samples(&mut self) {
        self.cpu.clear_audio_samples();
    }

    #[inline]
    pub fn clock_rate(&mut self) -> f32 {
        self.cpu.clock_rate()
    }

    /// Steps the control deck one CPU clock.
    ///
    /// # Errors
    ///
    /// If CPU encounteres an invalid opcode, an error is returned.
    pub fn clock_instr(&mut self) -> Result<ControlFlow<usize, usize>> {
        let cycles = self.clock();
        if self.cpu_corrupted() {
            Err(anyhow!("cpu corrupted"))
        } else {
            Ok(ControlFlow::Continue(cycles))
        }
    }

    /// Steps the control deck the number of seconds.
    ///
    /// # Errors
    ///
    /// If CPU encounteres an invalid opcode, an error is returned.
    pub fn clock_seconds(&mut self, seconds: f32) -> Result<ControlFlow<usize, usize>> {
        self.cycles_remaining += self.clock_rate() * seconds;
        let mut total_cycles = 0;
        while self.cycles_remaining > 0.0 {
            match self.clock_instr()? {
                ControlFlow::Break(cycles) => {
                    total_cycles += cycles;
                    self.cycles_remaining -= cycles as f32;
                    return Ok(ControlFlow::Break(total_cycles));
                }
                ControlFlow::Continue(cycles) => {
                    total_cycles += cycles;
                    self.cycles_remaining -= cycles as f32;
                }
            }
        }
        Ok(ControlFlow::Continue(total_cycles))
    }

    /// Steps the control deck the number of seconds with an inspection function, executed on every
    /// CPU clock.
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn clock_seconds_inspect<F>(
        &mut self,
        seconds: f32,
        mut inspect: F,
    ) -> Result<ControlFlow<usize, usize>>
        where
            F: FnMut(&mut Cpu),
    {
        self.cycles_remaining += self.clock_rate() * seconds;
        let mut total_cycles = 0;
        while self.cycles_remaining > 0.0 {
            let cycles = self.cpu.clock_inspect(&mut inspect);
            total_cycles += cycles;
            self.cycles_remaining -= cycles as f32;
        }
        Ok(ControlFlow::Continue(total_cycles))
    }

    /// Steps the control deck an entire frame
    ///
    /// # Errors
    ///
    /// If CPU encounteres an invalid opcode, an error is returned.
    pub fn clock_frame(&mut self) -> Result<ControlFlow<usize, usize>> {
        let mut total_cycles = 0;
        let frame = self.frame_number();
        while frame == self.frame_number() {
            match self.clock_instr()? {
                ControlFlow::Break(cycles) => {
                    total_cycles += cycles;
                    return Ok(ControlFlow::Break(total_cycles));
                }
                ControlFlow::Continue(cycles) => {
                    total_cycles += cycles;
                }
            }
        }
        Ok(ControlFlow::Continue(total_cycles))
    }

    /// Steps the control deck a single scanline.
    ///
    /// # Errors
    ///
    /// If CPU encounteres an invalid opcode, an error is returned.
    pub fn clock_scanline(&mut self) -> Result<ControlFlow<usize, usize>> {
        let current_scanline = self.cpu.ppu_scanline();
        let mut total_cycles = 0;
        while current_scanline == self.cpu.ppu_scanline() {
            match self.clock_instr()? {
                ControlFlow::Break(cycles) => {
                    total_cycles += cycles;
                    return Ok(ControlFlow::Break(total_cycles));
                }
                ControlFlow::Continue(cycles) => {
                    total_cycles += cycles;
                }
            }
        }
        Ok(ControlFlow::Continue(total_cycles))
    }

    /// Returns whether the CPU is corrupted or not.
    #[inline]
    #[must_use]
    pub const fn cpu_corrupted(&self) -> bool {
        self.cpu.corrupted()
    }

    #[inline]
    pub const fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    #[inline]
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    #[inline]
    pub const fn ppu(&self) -> &Ppu {
        self.cpu.ppu()
    }

    #[inline]
    pub fn ppu_mut(&mut self) -> &mut Ppu {
        self.cpu.ppu_mut()
    }

    #[inline]
    pub const fn apu(&self) -> &Apu {
        self.cpu.apu()
    }

    #[inline]
    pub const fn mapper(&self) -> &Mapper {
        self.cpu.mapper()
    }

    #[inline]
    pub fn mapper_mut(&mut self) -> &mut Mapper {
        self.cpu.mapper_mut()
    }

    /// Returns whether Four Score is enabled.
    #[inline]
    pub const fn four_player(&self) -> FourPlayer {
        self.cpu.four_player()
    }

    /// Enable/Disable Four Score for 4-player controllers.
    #[inline]
    pub fn set_four_player(&mut self, four_player: FourPlayer) {
        self.cpu.set_four_player(four_player);
    }

    /// Enable/Disable cycle accurate mode
    #[inline]
    pub fn set_cycle_accurate(&mut self, enabled: bool) {
        self.cpu.set_cycle_accurate(enabled);
    }

    /// Returns a mutable reference to a joypad.
    #[inline]
    pub fn joypad_mut(&mut self, slot: Player) -> &mut Joypad {
        self.cpu.joypad_mut(slot)
    }

    /// Returns the zapper aiming position for the given controller slot.
    #[inline]
    #[must_use]
    pub const fn zapper_pos(&self) -> (i32, i32) {
        let zapper = self.cpu.zapper();
        (zapper.x(), zapper.y())
    }

    /// Trigger Zapper gun for a given controller slot.
    #[inline]
    pub fn trigger_zapper(&mut self) {
        self.cpu.zapper_mut().trigger();
    }

    /// Aim Zapper gun for a given controller slot.
    #[inline]
    pub fn aim_zapper(&mut self, x: i32, y: i32) {
        self.cpu.zapper_mut().aim(x, y);
    }

    /// Set the image filter for video output.
    #[inline]
    pub fn set_filter(&mut self, filter: VideoFilter) {
        self.video.set_filter(filter);
    }

    /// Enable Zapper gun.
    #[inline]
    pub fn connect_zapper(&mut self, enabled: bool) {
        self.cpu.connect_zapper(enabled);
    }

    /// Add NES Game Genie codes.
    ///
    /// # Errors
    ///
    /// If genie code is invalid, an error is returned.
    #[inline]
    pub fn add_genie_code(&mut self, genie_code: String) -> Result<()> {
        self.cpu.add_genie_code(genie_code)
    }

    #[inline]
    pub fn remove_genie_code(&mut self, genie_code: &str) {
        self.cpu.remove_genie_code(genie_code);
    }

    /// Returns whether a given API audio channel is enabled.
    #[inline]
    #[must_use]
    pub const fn channel_enabled(&self, channel: Channel) -> bool {
        self.cpu.audio_channel_enabled(channel)
    }

    /// Toggle one of the APU audio channels.
    #[inline]
    pub fn toggle_channel(&mut self, channel: Channel) {
        self.cpu.toggle_audio_channel(channel);
    }

    /// Is control deck running.
    #[inline]
    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.running
    }
}

impl Clock for ControlDeck {
    /// Steps the control deck a single clock cycle.
    fn clock(&mut self) -> usize {
        self.cpu.clock()
    }
}

impl Regional for ControlDeck {
    /// Get the NES format for the emulation.
    #[inline]
    fn region(&self) -> NesRegion {
        self.cpu.region()
    }

    /// Set the NES format for the emulation.
    #[inline]
    fn set_region(&mut self, region: NesRegion) {
        self.cpu.set_region(region);
        self.cpu.set_region(region);
    }
}

impl Reset for ControlDeck {
    /// Resets the console.
    fn reset(&mut self, kind: ResetKind) {
        self.cpu.reset(kind);
        self.running = true;
    }
}
