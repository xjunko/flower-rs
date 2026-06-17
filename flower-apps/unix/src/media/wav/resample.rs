pub fn resample_linear_bits(
    input_bytes: &[u8],
    input_bits: u16,
    input_rate: u32,
    output_rate: u32,
    output: &mut alloc::vec::Vec<i16>,
) {
    if input_bytes.len() < 2 || input_rate == 0 || output_rate == 0 {
        return;
    }

    let mut input_samples = alloc::vec::Vec::<i16>::new();
    match input_bits {
        8 => {
            for &b in input_bytes {
                input_samples.push((b as i16 - 128) << 8);
            }
        },
        16 => {
            let len = input_bytes.len() / 2;
            let samples: &[i16] = unsafe {
                core::slice::from_raw_parts(
                    input_bytes.as_ptr() as *const i16,
                    len,
                )
            };
            input_samples.extend_from_slice(samples);
        },
        24 => {
            let len = input_bytes.len() / 3;
            for i in 0..len {
                let offset = i * 3;
                let val = ((input_bytes[offset] as i32)
                    | ((input_bytes[offset + 1] as i32) << 8)
                    | ((input_bytes[offset + 2] as i32) << 16))
                    << 8;
                input_samples.push((val >> 16) as i16);
            }
        },
        32 => {
            let len = input_bytes.len() / 4;
            for i in 0..len {
                let offset = i * 4;
                let val = (input_bytes[offset] as i32)
                    | ((input_bytes[offset + 1] as i32) << 8)
                    | ((input_bytes[offset + 2] as i32) << 16)
                    | ((input_bytes[offset + 3] as i32) << 24);
                input_samples.push((val >> 16) as i16);
            }
        },
        _ => {
            return;
        },
    }

    if input_samples.len() < 2 {
        return;
    }

    let mut pos: u64 = 0;
    let step: u64 = ((input_rate as u64) << 16) / (output_rate as u64);
    let max_pos = ((input_samples.len() - 1) as u64) << 16;

    while pos < max_pos {
        let idx = (pos >> 16) as usize;
        let frac = (pos & 0xFFFF) as i32;

        let s0 = input_samples[idx] as i32;
        let s1 = input_samples[idx + 1] as i32;

        let sample = ((s0 * (0x10000 - frac) + s1 * frac) >> 16) as i16;
        output.push(sample);

        pos += step;
    }
}
