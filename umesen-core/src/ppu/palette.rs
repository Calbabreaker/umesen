#[derive(Clone)]
pub struct Palette([[u8; 3]; 64]);

impl Default for Palette {
    fn default() -> Self {
        Self::from_pal(&include_bytes!("default.pal")[..]).unwrap()
    }
}

impl Palette {
    pub fn from_pal(mut bytes: impl std::io::Read) -> std::io::Result<Self> {
        let mut palette = Palette([[0; 3]; 64]);

        for palette_byte in &mut palette.0 {
            bytes.read_exact(palette_byte)?;
        }

        Ok(palette)
    }

    /// Gets the RGBA color value of the index in the palette
    pub fn get(&self, index: u8) -> [u8; 3] {
        self.0[index as usize % self.0.len()]
    }
}

#[cfg(test)]
mod test {
    use crate::ppu::palette::Palette;

    #[test]
    pub fn parse_correct() {
        let palette = Palette::default();
        assert_eq!(palette.get(0), [0x62, 0x62, 0x62]);
        assert_eq!(palette.get(1), [0x00, 0x2e, 0x98]);
        assert_eq!(palette.get(2), [0x0c, 0x11, 0xc2]);
    }
}
