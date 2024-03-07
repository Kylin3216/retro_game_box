use anyhow::{Error,Result,anyhow};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[must_use]
pub enum NesRegion {
    #[default]
    Ntsc,
    Pal,
    Dendy,
}

impl NesRegion {
    pub const fn as_slice() -> &'static [Self] {
        &[NesRegion::Ntsc, NesRegion::Pal, NesRegion::Dendy]
    }
}

impl AsRef<str> for NesRegion {
    fn as_ref(&self) -> &str {
        match self {
            Self::Ntsc => "NTSC",
            Self::Pal => "PAL",
            Self::Dendy => "Dendy",
        }
    }
}

impl TryFrom<&str> for NesRegion {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "NTSC" => Ok(Self::Ntsc),
            "PAL" => Ok(Self::Pal),
            "Dendy" => Ok(Self::Dendy),
            _ => Err(anyhow!("invalid nes region")),
        }
    }
}

impl From<usize> for NesRegion {
    fn from(value: usize) -> Self {
        match value {
            1 => Self::Pal,
            2 => Self::Dendy,
            _ => Self::Ntsc,
        }
    }
}

#[enum_dispatch(Mapper)]
pub trait Regional {
    fn region(&self) -> NesRegion {
        NesRegion::default()
    }
    fn set_region(&mut self, _region: NesRegion) {}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[must_use]
pub enum ResetKind {
    Soft,
    Hard,
}

#[enum_dispatch(Mapper)]
pub trait Reset {
    fn reset(&mut self, _kind: ResetKind) {}
}

#[enum_dispatch(Mapper)]
pub trait Clock {
    fn clock(&mut self) -> usize {
        0
    }
    fn clock_to(&mut self, _clocks: u64) {}
}

#[macro_export]
macro_rules! btree_map {
    { $($key:expr => $value:expr),* $(,)? } => {{
        let mut m = alloc::collections::BTreeMap::new();
        $(
            m.insert($key, $value);
        )*
        m
    }};
    ($hm:ident, { $($key:expr => $value:expr),* $(,)? } ) => ({
        $(
            $hm.insert($key, $value);
        )*
    });
}