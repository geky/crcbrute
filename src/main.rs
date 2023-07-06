// Search for good CRC polynomials
//
// This only looks at primitive even-parity polynomials, and only looks
// for the best Hamming distance 5, since this should give good properties
// for 1-5 bit errors.
//
// Based on hdlen.cpp by Philip Koopman:
// http://users.ece.cmu.edu/~koopman/crc/hdlen.html
//

#![allow(dead_code)]

use structopt::StructOpt;
use core::num;
use core::str::FromStr;

// hardware polynomial multiplication
mod pmul;
use pmul::pmul32;

// software polynomial division
fn pdivmod64(a: u64, b: u64) -> Option<(u64, u64)> {
    if b == 0 {
        return None;
    }

    let mut q = 0;
    let mut r = a;
    while r.leading_zeros() <= b.leading_zeros() {
        q ^= 1 << (b.leading_zeros()-r.leading_zeros());
        r ^= b << (b.leading_zeros()-r.leading_zeros());
    }
    Some((q, r))
}

fn pdiv64(a: u64, b: u64) -> u64 {
    pdivmod64(a, b).unwrap().0
}

fn pmod64(a: u64, b: u64) -> u64 {
    pdivmod64(a, b).unwrap().1
}


// CRC implementation using Barret reduction
struct Crc32 {
    p: u64,
    b: u32,
    p_r: u32,
    b_r: u32,
}

impl Crc32 {
    fn new(p: u64) -> Crc32 {
        // calculate our barret constant
        let b = pdiv64(p << 32, p) as u32;
        // and bit-reversed representations
        let p_r = (p as u32).reverse_bits();
        let b_r = b.reverse_bits();

        Crc32{p, b, p_r, b_r}
    }

    fn crc32(&self, crc: u32, data: &[u8]) -> u32 {
        // bit invert
        let mut crc = crc ^ 0xffffffff;

        // operate on 4-byte chunks first
        let mut words = data.chunks_exact(4);
        for word in &mut words {
            crc ^= u32::from_le_bytes(<[u8; 4]>::try_from(word).unwrap());
            let (lo, _) = pmul32(crc, self.b_r);
            let (lo, hi) = pmul32((lo << 1) ^ crc, self.p_r);
            crc = (hi << 1) | (lo >> 31);
        }

        // now clean up any remaining bytes
        for b in words.remainder() {
            crc ^= *b as u32;
            let (lo, _) = pmul32(crc << 24, self.b_r);
            let (lo, hi) = pmul32((lo << 1) ^ (crc << 24), self.p_r);
            crc = (crc >> 8) ^ ((hi << 1) | (lo >> 31));
        }

        // bit invert
        crc ^ 0xffffffff
    }
}



// more parsers
fn parse_u32(s: &str) -> Result<u32, num::ParseIntError> {
    if s.starts_with("0x") {
        Ok(u32::from_str_radix(&s[2..], 16)?)
    } else if s.starts_with("0o") {
        Ok(u32::from_str_radix(&s[2..], 8)?)
    } else if s.starts_with("0b") {
        Ok(u32::from_str_radix(&s[2..], 2)?)
    } else {
        Ok(u32::from_str(s)?)
    }
}

fn parse_u64(s: &str) -> Result<u64, num::ParseIntError> {
    if s.starts_with("0x") {
        Ok(u64::from_str_radix(&s[2..], 16)?)
    } else if s.starts_with("0o") {
        Ok(u64::from_str_radix(&s[2..], 8)?)
    } else if s.starts_with("0b") {
        Ok(u64::from_str_radix(&s[2..], 2)?)
    } else {
        Ok(u64::from_str(s)?)
    }
}

// CLI arguments
#[derive(Debug, StructOpt)]
#[structopt(rename_all="kebab")]
struct Opt {
    /// Prefix of the message we want to find a specific CRC value for
    prefix: String,

    /// CRC value we want
    #[structopt(parse(try_from_str=parse_u32))]
    target: u32,

    /// CRC polynomial, currently limited to 32-bits
    #[structopt(short, long,
        default_value="0x11edc6f41",
        parse(try_from_str=parse_u64)
    )]
    polynomial: u64,

    /// Limit results to ascii characters, note this doubles the brute
    /// force suffix
    #[structopt(long)]
    ascii: bool,
}

// entry point
fn main() {
    let opt = Opt::from_args();

    // create our CRC
    let crc32 = Crc32::new(opt.polynomial);

    // find the CRC of our prefix
    let mut x = crc32.crc32(0, &opt.prefix.as_bytes());
    // find CRC of just our implicit xor
    let mut c = 0;
    // + space for suffix
    if opt.ascii {
        x = crc32.crc32(x, &[0, 0, 0, 0, 0, 0, 0, 0]);
        c = crc32.crc32(c, &[0, 0, 0, 0, 0, 0, 0, 0]);
    } else {
        x = crc32.crc32(x, &[0, 0, 0, 0]);
        c = crc32.crc32(c, &[0, 0, 0, 0]);
    }

    // this xor is our target value
    let target = x ^ opt.target ^ c;

    if opt.ascii {
        // brute force find a 64-bit suffix that makes our CRC work, skipping
        // any non-ascii and non-control characters
        //
        // since DEL (0x7f) is a control character, and space (0x20) is sort of
        // a control character, we limit our characters to H..=W (0x48..=0x57)
        // and h..=w (0x68..=0x77). This gives us 5 bits per per character to
        // work with.
        for i in 0x00_0000_0000u64 ..= 0xff_ffff_ffffu64 {
            // convert into a guaranteed ascii representation
            // first get all bits into the right position
            let i = ((i << 12) & 0x000f_ffff_0000_0000) | (i & 0x0000_0000_000f_ffff);
            let i = ((i <<  6) & 0x03ff_0000_03ff_0000) | (i & 0x0000_03ff_0000_03ff);
            let i = ((i <<  3) & 0x1f00_1f00_1f00_1f00) | (i & 0x001f_001f_001f_001f);
            let i = ((i <<  1) & 0x2020_2020_2020_2020) | (i & 0x0f0f_0f0f_0f0f_0f0f);
            // and then add to array of 0x48s
            let i = i + 0x48_48_48_48_48_48_48_48;

            if crc32.crc32(0, &i.to_le_bytes()) == target {
                for b in
                    opt.prefix.as_bytes().iter().copied()
                        .chain(i.to_le_bytes())
                {
                    if b >= ' ' as u8 && b <= '~' as u8 {
                        print!("{}", b as char);
                    } else {
                        print!("\\x{:02x}", b);
                    }
                }
                println!();

                // validate that the checksum matches
                assert_eq!(
                    crc32.crc32(crc32.crc32(0,
                        opt.prefix.as_bytes()),
                        &i.to_le_bytes()),
                    opt.target
                );
                break;
            }
        }
    } else {
        // brute force find a 32-bit suffix that makes our CRC work
        for i in 0x0000_0000u32 ..= 0xffff_ffffu32 {
            if crc32.crc32(0, &i.to_le_bytes()) == target {
                for b in
                    opt.prefix.as_bytes().iter().copied()
                        .chain(i.to_le_bytes())
                {
                    if b >= ' ' as u8 && b <= '~' as u8 {
                        print!("{}", b as char);
                    } else {
                        print!("\\x{:02x}", b);
                    }
                }
                println!();

                // validate that the checksum matches
                assert_eq!(
                    crc32.crc32(crc32.crc32(0,
                        opt.prefix.as_bytes()),
                        &i.to_le_bytes()),
                    opt.target
                );
                break;
            }
        }
    }
}
