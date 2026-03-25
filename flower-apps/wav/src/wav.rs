pub struct Wav<'a> {
    pub channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub data: &'a [u8],
}

fn read_u16_le(bytes: &[u8]) -> u16 { u16::from_le_bytes([bytes[0], bytes[1]]) }

fn read_u32_le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

pub fn parse_wav<'a>(data: &'a [u8]) -> Option<Wav<'a>> {
    if data.len() < 44 {
        return None;
    }

    if &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return None;
    }

    let mut offset = 12;

    let mut channels = 0;
    let mut sample_rate = 0;
    let mut bits_per_sample = 0;
    let mut pcm_data: Option<&[u8]> = None;

    while offset + 8 <= data.len() {
        let chunk_id = &data[offset..offset + 4];
        let chunk_size = read_u32_le(&data[offset + 4..offset + 8]) as usize;

        offset += 8;

        if offset + chunk_size > data.len() {
            return None;
        }

        match chunk_id {
            b"fmt " => {
                let audio_format = read_u16_le(&data[offset..offset + 2]);
                channels = read_u16_le(&data[offset + 2..offset + 4]);
                sample_rate = read_u32_le(&data[offset + 4..offset + 8]);
                bits_per_sample = read_u16_le(&data[offset + 14..offset + 16]);

                // NOTE: only support PCM (format = 1)
                if audio_format != 1 {
                    return None;
                }
            },
            b"data" => {
                pcm_data = Some(&data[offset..offset + chunk_size]);
            },
            _ => {},
        }

        offset += chunk_size;

        if chunk_size % 2 == 1 {
            offset += 1;
        }
    }

    Some(Wav { channels, sample_rate, bits_per_sample, data: pcm_data? })
}
