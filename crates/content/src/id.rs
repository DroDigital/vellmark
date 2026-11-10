/// FNV-1a, 64 bit, usable in constants.
pub const fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut i = 0;
    while i < bytes.len() {
        h ^= bytes[i] as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
        i += 1;
    }
    h
}

/// What an asset is. The number is part of the pack format and is never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u32)]
pub enum AssetKind {
    StringTable = 1,
    Tile = 2,
    Texture = 3,
    Mesh = 4,
    GlyphAtlas = 5,
    TextPage = 6,
    WordSheet = 7,
    Scene = 8,
    Sound = 9,
    Table = 10,
}

impl AssetKind {
    pub fn from_u32(v: u32) -> Option<Self> {
        Some(match v {
            1 => Self::StringTable,
            2 => Self::Tile,
            3 => Self::Texture,
            4 => Self::Mesh,
            5 => Self::GlyphAtlas,
            6 => Self::TextPage,
            7 => Self::WordSheet,
            8 => Self::Scene,
            9 => Self::Sound,
            10 => Self::Table,
            _ => return None,
        })
    }
}

/// The permanent ID of an asset: the hash of its authored key, such as
/// `place.tithe_hill.stone_07`. A key is never renamed once it has shipped, so an update
/// can never break a save that refers to it. No key is stored in a pack.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetId(pub u64);

impl AssetId {
    pub const fn from_key(key: &str) -> Self {
        Self(fnv1a64(key.as_bytes()))
    }
}

/// The permanent ID of a piece of player-facing text. Code holds these and never the
/// text itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StringId(pub u64);

impl StringId {
    pub const fn from_key(key: &str) -> Self {
        Self(fnv1a64(key.as_bytes()))
    }
}

/// `sid!("ui.title")` is the [`StringId`] of that key, computed when the game is
/// compiled. The harness checks that every key written this way exists in the tables.
#[macro_export]
macro_rules! sid {
    ($key:literal) => {{
        const ID: $crate::StringId = $crate::StringId::from_key($key);
        ID
    }};
}
