bitflags::bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Attributes: u8 {
        const PALLETTE = 0b11;
        const BEHIND = 1 << 5;
        const FLIP_HORIZONTAL = 1 << 6;
        const FLIP_VERTICAL = 1 << 7;
    }
}

impl Attributes {
    pub fn palette(&self) -> u8 {
        (*self & Attributes::PALLETTE).bits()
    }
}

impl std::fmt::Display for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flag_map = [
            (Attributes::BEHIND, "B"),
            (Attributes::FLIP_HORIZONTAL, "H"),
            (Attributes::FLIP_VERTICAL, "V"),
        ];
        for (flag, name) in flag_map {
            write!(f, "{} ", if self.contains(flag) { name } else { "-" })?;
        }
        write!(f, "{}", self.palette())?;
        Ok(())
    }
}

pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub tile_number: u8,
    pub attributes: Attributes,
}

impl Sprite {
    pub fn new(oam: &[u8]) -> Self {
        debug_assert_eq!(oam.len(), 4);
        Self {
            x: oam[3],
            y: oam[0],
            tile_number: oam[1],
            attributes: Attributes::from_bits_truncate(oam[2]),
        }
    }
}
