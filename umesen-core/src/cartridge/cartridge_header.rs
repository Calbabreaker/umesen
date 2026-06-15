use crate::{NesParseError, cartridge::KB};

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum Mirroring {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct CartridgeHeader {
    pub mapper_id: u16,
    pub prg_rom_size: usize,
    pub prg_ram_size: usize,
    pub chr_mem_size: usize,
    pub chr_mem_is_rom: bool,
    pub has_trainer: bool,
    pub mirroring: Mirroring,
    pub is_v2: bool,
    pub has_volatile: bool,
}

impl CartridgeHeader {
    pub const TRAINER_SIZE: usize = 512;
    pub const CHR_BANK_SIZE: usize = 8 * KB;
    pub const PRG_BANK_SIZE: usize = 16 * KB;

    pub fn from_nes(data: [u8; 16]) -> Result<Self, NesParseError> {
        if &data[0..4] != b"NES\x1a" {
            let magic_number = String::from_utf8_lossy(&data[0..4]).to_string();
            return Err(NesParseError::InvalidMagicNumber(magic_number));
        }

        let is_v2 = data[7] & 0b0000_1100 == 0b0000_1000;
        let mut mapper_id = ((data[6] >> 4) | (data[7] & 0xf0)) as u16;

        let prg_ram_size = if is_v2 {
            get_shifted_size(data[10] & 0x0f)
        } else {
            let units = data[8].max(1);
            (units as usize) * 8 * KB
        };

        let chr_ram_size = {
            if is_v2 {
                get_shifted_size(data[11] & 0x0f)
            } else {
                8 * 1024
            }
        };

        let mut prg_rom_size = (data[4] as usize) * Self::PRG_BANK_SIZE;
        let mut chr_rom_size = (data[5] as usize) * Self::CHR_BANK_SIZE;

        if is_v2 {
            prg_rom_size |= (data[9] as usize & 0x0f) << 8;
            chr_rom_size |= (data[9] as usize & 0xf0) << 4;
            mapper_id |= (data[8] as u16 & 0x0f) << 8;
        }

        Ok(Self {
            prg_rom_size,
            mapper_id,
            mirroring: if data[6] & 0b000_0001 == 0 {
                Mirroring::Horizontal
            } else {
                Mirroring::Vertical
            },
            has_volatile: data[6] & 0b0000_0010 != 0,
            has_trainer: data[6] & 0b0000_0100 != 0,
            chr_mem_size: if chr_rom_size == 0 {
                chr_ram_size
            } else {
                chr_rom_size
            },
            chr_mem_is_rom: chr_rom_size != 0,
            prg_ram_size,
            is_v2,
        })
    }
}

fn get_shifted_size(shift_count: u8) -> usize {
    if shift_count == 0 {
        0
    } else {
        64 << (shift_count as usize)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_correctly() {
        let header = CartridgeHeader::from_nes([
            0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ])
        .unwrap();

        assert_eq!(
            header,
            CartridgeHeader {
                has_volatile: false,
                mapper_id: 3,
                mirroring: Mirroring::Vertical,
                prg_rom_size: 32 * 1024,
                chr_mem_size: 8 * 1024,
                prg_ram_size: 8 * 1024,
                has_trainer: false,
                chr_mem_is_rom: true,
                is_v2: false
            }
        )
    }
}
