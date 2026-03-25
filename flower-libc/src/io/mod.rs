use crate::print;
use crate::sys::fs::{self, open};

pub fn getch() -> u8 {
    let kb = open(b"/dev/keyboard\0", 0, 0);
    if kb < 0 {
        return 0;
    }

    let mut c = [0u8; 1];
    loop {
        let _ = fs::read(kb as u64, &mut c);
        if c[0] != 0 {
            fs::close(kb as u64);
            return c[0];
        }
    }
}

pub fn read_line(buf: &mut [u8]) -> usize {
    let mut pos = 0;
    loop {
        let c = getch();

        match c {
            b'\n' => {
                print!("\n");
                return pos;
            },
            b'\x08' => {
                if pos > 0 {
                    pos -= 1;
                    print!("\x08 \x08");
                }
            },
            32..126 => {
                if pos < buf.len() {
                    buf[pos] = c;
                    pos += 1;
                    print!("{}", c as char);
                }
            },
            _ => break,
        }
    }
    0
}
