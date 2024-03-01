use alloc::vec;
use alloc::vec::Vec;
use super::Ppu;
use crate::{
    common::{NesRegion, Regional, Reset, ResetKind},
    mapper::{Mapped, MappedRead, MappedWrite, Mapper, MemMap},
    mem::{Access, Mem},
    ppu::Mirroring,
};
use serde::{Deserialize, Serialize};

pub trait PpuAddr {
    /// Returns whether this value can be used to fetch a nametable attribute byte.
    fn is_attr(&self) -> bool;
    fn is_palette(&self) -> bool;
}

impl PpuAddr for u16 {
    fn is_attr(&self) -> bool {
        (*self & 0x03FF) >= 0x03C0
    }

    fn is_palette(&self) -> bool {
        *self >= 0x3F00
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[must_use]
pub struct PpuBus {
    pub mapper: Mapper,
    pub ciram: Vec<u8>, // $2007 PPUDATA
    pub palette: [u8; Self::PALETTE_SIZE],
    pub chr: Vec<u8>,
    pub exram: Vec<u8>,
    mirror_shift: usize,
    open_bus: u8,
}

impl Default for PpuBus {
    fn default() -> Self {
        Self::new()
    }
}

impl PpuBus {
    const VRAM_SIZE: usize = 0x0800; // Two 1k Nametables
    const PALETTE_SIZE: usize = 32; // 32 possible colors at a time

    pub fn new() -> Self {
        Self {
            mapper: Mapper::none(),
            ciram: vec![0x00; Self::VRAM_SIZE],
            palette: [0x00; Self::PALETTE_SIZE],
            chr: vec![],
            exram: vec![],
            mirror_shift: Mirroring::default() as usize,
            open_bus: 0x00,
        }
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mapper.mirroring()
    }

    pub fn update_mirroring(&mut self) {
        self.mirror_shift = self.mapper.mirroring() as usize;
    }

    pub fn load_chr(&mut self, chr: Vec<u8>) {
        self.chr = chr;
    }

    pub fn load_ex_ram(&mut self, ex_ram: Vec<u8>) {
        self.exram = ex_ram;
    }

    // Maps addresses to nametable pages based on mirroring mode
    //
    // Vram:            [ A ] [ B ]
    //
    // Horizontal:      [ A ] [ a ]
    //                  [ B ] [ b ]
    //
    // Vertical:        [ A ] [ B ]
    //                  [ a ] [ b ]
    //
    // Single Screen A: [ A ] [ a ]
    //                  [ a ] [ a ]
    //
    // Single Screen B: [ b ] [ B ]
    //                  [ b ] [ b ]
    //
    // Fourscreen should not use this method and instead should rely on mapper translation.

    const fn ciram_mirror(&self, addr: usize) -> usize {
        let nametable = (addr >> self.mirror_shift) & (Ppu::NT_SIZE as usize);
        nametable | (!nametable & addr & 0x03FF)
    }

    const fn palette_mirror(&self, addr: usize) -> usize {
        addr & 0x001F
    }
}

impl Mem for PpuBus {
    fn read(&mut self, addr: u16, _access: Access) -> u8 {
        let val = match addr {
            0x2000..=0x3EFF => match self.mapper.map_read(addr) {
                MappedRead::PpuRam => self.ciram[self.ciram_mirror(addr as usize)],
                MappedRead::CIRam(addr) => self.ciram[addr & 0x07FF],
                MappedRead::ExRam(addr) => self.exram[addr],
                MappedRead::Data(data) => data,
                MappedRead::Chr(mapped) => {
                    panic!("unexpected mapped CHR read at ${addr:04X} for ${mapped:04X}")
                }
                MappedRead::PrgRom(mapped) => {
                    panic!("unexpected mapped PRG-ROM read at ${addr:04X} ${mapped:04X}")
                }
                MappedRead::PrgRam(mapped) => {
                    panic!("unexpected mapped PRG-RAM read at ${addr:04X} ${mapped:04X}")
                }
            },
            0x0000..=0x1FFF => {
                let addr = if let MappedRead::Chr(addr) = self.mapper.map_read(addr) {
                    addr
                } else {
                    addr.into()
                };
                self.chr[addr]
            }
            // Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => self.read(addr - 0x10, _access),
            0x3F00..=0x3FFF => self.palette[self.palette_mirror(addr as usize)],
            _ => {
                log::error!("unexpected PPU memory access at ${:04X}", addr);
                0x00
            }
        };
        self.open_bus = val;
        val
    }

    fn peek(&self, addr: u16, _access: Access) -> u8 {
        match addr {
            0x2000..=0x3EFF => match self.mapper.map_peek(addr) {
                MappedRead::PpuRam => self.ciram[self.ciram_mirror(addr as usize)],
                MappedRead::CIRam(addr) => self.ciram[addr & 0x07FF],
                MappedRead::ExRam(addr) => self.exram[addr],
                MappedRead::Data(data) => data,
                MappedRead::Chr(mapped) => {
                    panic!("unexpected mapped CHR read at ${addr:04X} for ${mapped:04X}")
                }
                MappedRead::PrgRom(mapped) => {
                    panic!("unexpected mapped PRG-ROM read at ${addr:04X} ${mapped:04X}")
                }
                MappedRead::PrgRam(mapped) => {
                    panic!("unexpected mapped PRG-RAM read at ${addr:04X} ${mapped:04X}")
                }
            },
            0x0000..=0x1FFF => {
                let addr = if let MappedRead::Chr(addr) = self.mapper.map_peek(addr) {
                    addr
                } else {
                    addr.into()
                };
                self.chr[addr]
            }
            0x3F00..=0x3FFF => self.palette[self.palette_mirror(addr as usize)],
            _ => {
                log::error!("unexpected PPU memory access at ${:04X}", addr);
                0x00
            }
        }
    }

    fn write(&mut self, addr: u16, val: u8, _access: Access) {
        match addr {
            0x2000..=0x3EFF => match self.mapper.map_write(addr, val) {
                MappedWrite::PpuRam => {
                    let addr = self.ciram_mirror(addr as usize);
                    self.ciram[addr] = val;
                }
                MappedWrite::CIRam(addr, val) => self.ciram[addr & 0x07FF] = val,
                MappedWrite::ExRam(addr, val) => self.exram[addr] = val,
                MappedWrite::Chr(mapped, val) => {
                    panic!("unexpected mapped CHR write at ${addr:04X} for ${mapped:04X} with ${val:02X}");
                }
                MappedWrite::PrgRam(mapped, val) => {
                    panic!("unexpected mapped PRG-RAM write at ${addr:04X} for ${mapped:04X} with ${val:02X}");
                }
                MappedWrite::PrgRamProtect(val) => {
                    panic!("unexpected mapped PRG-RAM Protect write at ${addr:04X} with {val}");
                }
                MappedWrite::None => (),
            },
            0x0000..=0x1FFF => {
                if let MappedWrite::Chr(addr, val) = self.mapper.map_write(addr, val) {
                    self.chr[addr] = val;
                }
            }
            0x3F00..=0x3FFF => {
                self.palette[self.palette_mirror(addr as usize)] = val;
            }
            _ => log::error!("unexpected PPU memory access at ${:04X}", addr),
        }
        self.mapper.ppu_bus_write(addr, val);
        self.open_bus = val;
    }
}

impl Regional for PpuBus {
    fn region(&self) -> NesRegion {
        self.mapper.region()
    }

    fn set_region(&mut self, region: NesRegion) {
        self.mapper.set_region(region);
    }
}

impl Reset for PpuBus {
    fn reset(&mut self, kind: ResetKind) {
        self.open_bus = 0x00;
        self.mapper.reset(kind);
    }
}

impl core::fmt::Debug for PpuBus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PpuBus")
            .field("mapper", &self.mapper)
            .field("ciram_len", &self.ciram.len())
            .field("palette_len", &self.palette.len())
            .field("chr_len", &self.chr.len())
            .field("ex_ram_len", &self.exram.len())
            .field("open_bus", &self.open_bus)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ciram_mirror_horizontal() {
        let mut ppu_bus = PpuBus::new();
        ppu_bus.mirror_shift = Mirroring::Horizontal as usize;

        assert_eq!(ppu_bus.ciram_mirror(0x2000), 0x0000);
        assert_eq!(ppu_bus.ciram_mirror(0x2005), 0x0005);
        assert_eq!(ppu_bus.ciram_mirror(0x23FF), 0x03FF);
        assert_eq!(ppu_bus.ciram_mirror(0x2400), 0x0000);
        assert_eq!(ppu_bus.ciram_mirror(0x2405), 0x0005);
        assert_eq!(ppu_bus.ciram_mirror(0x27FF), 0x03FF);

        assert_eq!(ppu_bus.ciram_mirror(0x2800), 0x0400);
        assert_eq!(ppu_bus.ciram_mirror(0x2805), 0x0405);
        assert_eq!(ppu_bus.ciram_mirror(0x2BFF), 0x07FF);
        assert_eq!(ppu_bus.ciram_mirror(0x2C00), 0x0400);
        assert_eq!(ppu_bus.ciram_mirror(0x2C05), 0x0405);
        assert_eq!(ppu_bus.ciram_mirror(0x2FFF), 0x07FF);
    }

    #[test]
    fn ciram_mirror_vertical() {
        let mut ppu_bus = PpuBus::new();
        ppu_bus.mirror_shift = Mirroring::Vertical as usize;

        assert_eq!(ppu_bus.ciram_mirror(0x2000), 0x0000);
        assert_eq!(ppu_bus.ciram_mirror(0x2005), 0x0005);
        assert_eq!(ppu_bus.ciram_mirror(0x23FF), 0x03FF);
        assert_eq!(ppu_bus.ciram_mirror(0x2800), 0x0000);
        assert_eq!(ppu_bus.ciram_mirror(0x2805), 0x0005);
        assert_eq!(ppu_bus.ciram_mirror(0x2BFF), 0x03FF);

        assert_eq!(ppu_bus.ciram_mirror(0x2400), 0x0400);
        assert_eq!(ppu_bus.ciram_mirror(0x2405), 0x0405);
        assert_eq!(ppu_bus.ciram_mirror(0x27FF), 0x07FF);
        assert_eq!(ppu_bus.ciram_mirror(0x2C00), 0x0400);
        assert_eq!(ppu_bus.ciram_mirror(0x2C05), 0x0405);
        assert_eq!(ppu_bus.ciram_mirror(0x2FFF), 0x07FF);
    }

    #[test]
    fn ciram_mirror_single_screen_a() {
        let mut ppu_bus = PpuBus::new();
        ppu_bus.mirror_shift = Mirroring::SingleScreenA as usize;

        assert_eq!(ppu_bus.ciram_mirror(0x2000), 0x0000);
        assert_eq!(ppu_bus.ciram_mirror(0x2005), 0x0005);
        assert_eq!(ppu_bus.ciram_mirror(0x23FF), 0x03FF);
        assert_eq!(ppu_bus.ciram_mirror(0x2800), 0x0000);
        assert_eq!(ppu_bus.ciram_mirror(0x2805), 0x0005);
        assert_eq!(ppu_bus.ciram_mirror(0x2BFF), 0x03FF);
        assert_eq!(ppu_bus.ciram_mirror(0x2400), 0x0000);
        assert_eq!(ppu_bus.ciram_mirror(0x2405), 0x0005);
        assert_eq!(ppu_bus.ciram_mirror(0x27FF), 0x03FF);
        assert_eq!(ppu_bus.ciram_mirror(0x2C00), 0x0000);
        assert_eq!(ppu_bus.ciram_mirror(0x2C05), 0x0005);
        assert_eq!(ppu_bus.ciram_mirror(0x2FFF), 0x03FF);
    }

    #[test]
    fn ciram_mirror_single_screen_b() {
        let mut ppu_bus = PpuBus::new();
        ppu_bus.mirror_shift = Mirroring::SingleScreenB as usize;

        assert_eq!(ppu_bus.ciram_mirror(0x2000), 0x0400);
        assert_eq!(ppu_bus.ciram_mirror(0x2005), 0x0405);
        assert_eq!(ppu_bus.ciram_mirror(0x23FF), 0x07FF);
        assert_eq!(ppu_bus.ciram_mirror(0x2800), 0x0400);
        assert_eq!(ppu_bus.ciram_mirror(0x2805), 0x0405);
        assert_eq!(ppu_bus.ciram_mirror(0x2BFF), 0x07FF);
        assert_eq!(ppu_bus.ciram_mirror(0x2400), 0x0400);
        assert_eq!(ppu_bus.ciram_mirror(0x2405), 0x0405);
        assert_eq!(ppu_bus.ciram_mirror(0x27FF), 0x07FF);
        assert_eq!(ppu_bus.ciram_mirror(0x2C00), 0x0400);
        assert_eq!(ppu_bus.ciram_mirror(0x2C05), 0x0405);
        assert_eq!(ppu_bus.ciram_mirror(0x2FFF), 0x07FF);
    }
}