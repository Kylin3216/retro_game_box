use flutter_rust_bridge::frb;
pub use nes_core::common::NesRegion;
pub use nes_core::genie::GenieCode;
pub use nes_core::input::{Player, FourPlayer};
pub use nes_core::mem::RamState;
pub use nes_core::video::VideoFilter;


#[frb(mirror(Player))]
pub enum _Player {
    One,
    Two,
    Three,
    Four,
}

#[frb(mirror(VideoFilter))]
pub enum _VideoFilter {
    Pixellate,
    Ntsc,
}

#[frb(mirror(NesRegion))]
pub enum _NesRegion {
    Ntsc,
    Pal,
    Dendy,
}


#[frb(mirror(RamState))]
pub enum _RamState {
    AllZeros,
    AllOnes,
    Random,
}

#[frb(mirror(FourPlayer))]
pub enum _FourPlayer {
    Disabled,
    FourScore,
    Satellite,
}

#[frb(mirror(GenieCode))]
pub struct _GenieCode {
    code: String,
    addr: u16,
    data: u8,
    compare: Option<u8>,
}