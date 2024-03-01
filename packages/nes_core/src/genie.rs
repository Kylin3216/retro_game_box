use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::{btree_map, NesError, NesResult};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref GENIE_MAP: BTreeMap<char, u8> = {
       btree_map! {
        'A' => 0x0, 'P' => 0x1, 'Z' => 0x2, 'L' => 0x3, 'G' => 0x4, 'I' => 0x5, 'T' => 0x6,
        'Y' => 0x7, 'E' => 0x8, 'O' => 0x9, 'X' => 0xA, 'U' => 0xB, 'K' => 0xC, 'S' => 0xD,
        'V' => 0xE, 'N' => 0xF
        }
    };
}
/// Game Genie Code
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenieCode {
    code: String,
    addr: u16,
    data: u8,
    compare: Option<u8>,
}

impl GenieCode {
    /// Creates a new `GenieCode` instance.
    ///
    /// # Errors
    ///
    /// This function will return an error if the given code is not the correct format.
    pub fn new(code: String) -> NesResult<Self> {
        let hex = Self::parse(&code)?;
        let addr = 0x8000
            + (((u16::from(hex[3]) & 7) << 12)
            | ((u16::from(hex[5]) & 7) << 8)
            | ((u16::from(hex[4]) & 8) << 8)
            | ((u16::from(hex[2]) & 7) << 4)
            | ((u16::from(hex[1]) & 8) << 4)
            | (u16::from(hex[4]) & 7)
            | (u16::from(hex[3]) & 8));
        let data = if hex.len() == 6 {
            ((hex[1] & 7) << 4) | ((hex[0] & 8) << 4) | (hex[0] & 7) | (hex[5] & 8)
        } else {
            ((hex[1] & 7) << 4) | ((hex[0] & 8) << 4) | (hex[0] & 7) | (hex[7] & 8)
        };
        let compare = if hex.len() == 8 {
            Some(((hex[7] & 7) << 4) | ((hex[6] & 8) << 4) | (hex[6] & 7) | (hex[5] & 8))
        } else {
            None
        };
        Ok(Self {
            code,
            addr,
            data,
            compare,
        })
    }

    pub fn parse(code: &str) -> NesResult<Vec<u8>> {
        if code.len() != 6 && code.len() != 8 {
            return Err(NesError::new(format!(
                "invalid game genie code: {code}. Length must be 6 or 8 characters."
            )));
        }
        let mut hex: Vec<u8> = Vec::with_capacity(code.len());
        for s in code.chars() {
            if let Some(h) = GENIE_MAP.get(&s) {
                hex.push(*h);
            } else {
                return Err(NesError::new(format!(
                    "invalid game genie code: {code}. Invalid character: {s}"
                )));
            }
        }
        Ok(hex)
    }

    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    #[must_use]
    pub const fn addr(&self) -> u16 {
        self.addr
    }

    #[must_use]
    pub const fn read(&self, val: u8) -> u8 {
        if let Some(compare) = self.compare {
            if val == compare {
                self.data
            } else {
                val
            }
        } else {
            self.data
        }
    }
}

impl core::fmt::Display for GenieCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", &self.code)
    }
}
