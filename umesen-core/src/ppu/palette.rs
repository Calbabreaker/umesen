pub struct Palette([u32; 64]);

impl Default for Palette {
    fn default() -> Self {
        Self::from_pal(&include_bytes!("default.pal")[..]).unwrap()
    }
}

impl Palette {
    pub fn from_pal(mut bytes: impl std::io::Read) -> std::io::Result<Self> {
        let mut palette = Palette([0; 64]);

        for palette_byte in &mut palette.0 {
            let mut rgb_bytes = [0; 4];
            bytes.read_exact(&mut rgb_bytes[1..4])?;
            *palette_byte = u32::from_be_bytes(rgb_bytes);
        }

        Ok(palette)
    }

    pub fn get(&self, index: u8) -> u32 {
        self.0[index as usize % self.0.len()]
    }
}

#[cfg(test)]
mod test {
    use crate::ppu::palette::Palette;

    #[test]
    pub fn parse_correct() {
        let palette = Palette::default();
        assert_eq!(palette.get(0), 0x626262);
        assert_eq!(palette.get(1), 0x002e98);
        assert_eq!(palette.get(2), 0x0c11c2);
    }
}
