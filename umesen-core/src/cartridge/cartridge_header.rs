use crate::NesParseError;

#[derive(Default, Debug, PartialEq, Clone)]
pub struct CartridgeHeader {
    pub mapper_id: u8,
    pub prg_rom_size: usize,
    pub prg_ram_size: usize,
    pub chr_rom_size: usize,
    pub has_trainer: bool,
    pub is_v2: bool,
}

impl CartridgeHeader {
    pub fn from_nes(data: [u8; 16]) -> Result<Self, NesParseError> {
        if &data[0..4] != b"NES\x1a" {
            let magic_number = String::from_utf8_lossy(&data[0..4]).to_string();
            return Err(NesParseError::InvalidMagicNumber(magic_number));
        }

        let is_v2 = data[7] & 0b0011_0000 == 0b0001_0000;

        let prg_ram_size = if is_v2 {
            64 << ((data[10] & 0xf) as usize)
        } else {
            let units = data[8].max(1);
            (units as usize) * 8 * 1024
        };

        Ok(Self {
            // 16KiB per unit in header
            prg_rom_size: (data[4] as usize) * 16 * 1024,
            // 8KiB per unit in header
            chr_rom_size: (data[5] as usize) * 8 * 1024,
            mapper_id: data[6] >> 4 | data[7] & 0xf0,
            has_trainer: data[6] & 0b0010_0000 != 0,
            prg_ram_size,
            is_v2,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::cartridge::cartridge_header::CartridgeHeader;

    #[test]
    fn parse_correctly() {
        let header = CartridgeHeader::from_nes([
            0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ])
        .unwrap();

        assert_eq!(
            header,
            CartridgeHeader {
                mapper_id: 0,
                prg_rom_size: 32 * 1024,
                chr_rom_size: 8 * 1024,
                prg_ram_size: 8 * 1024,
                has_trainer: false,
                is_v2: false
            }
        )
    }
}