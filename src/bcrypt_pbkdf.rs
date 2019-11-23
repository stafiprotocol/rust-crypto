// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use blowfish::Blowfish;
use cryptoutil::{read_u32v_be, write_u32_be, write_u32_le};
use digest::Digest;
use sha2::Sha512;
use step_by::RangeExt;

fn bcrypt_hash(hpass: &[u8], hsalt: &[u8], output: &mut [u8; 32]) {
    let mut bf = Blowfish::init_state();
    bf.salted_expand_key(hsalt, hpass);

    for _ in 0..64 {
        bf.expand_key(hsalt);
        bf.expand_key(hpass);
    }

    let mut buf = [0u32; 8];
    read_u32v_be(&mut buf, b"OxychromaticBlowfishSwatDynamite");

    for i in (0..8).step_up(2) {
        for _ in 0..64 {
            let (l, r) = bf.encrypt(buf[i], buf[i + 1]);
            buf[i] = l;
            buf[i + 1] = r;
        }
    }

    for i in 0..8 {
        write_u32_le(&mut output[i * 4..(i + 1) * 4], buf[i]);
    }
}

pub fn bcrypt_pbkdf(password: &[u8], salt: &[u8], rounds: u32, output: &mut [u8]) {
    let mut hpass = [0u8; 64];

    //assert!(password.len() > 0);
    //assert!(salt.len() > 0);
    //assert!(rounds > 0);
    //assert!(output.len() > 0);
    //assert!(output.len() <= 1024);

    let nblocks = (output.len() + 31) / 32;

    let mut h = Sha512::new();
    h.input(password);
    h.result(&mut hpass);

    for block in 1..(nblocks + 1) {
        let mut count = [0u8; 4];
        let mut hsalt = [0u8; 64];
        let mut out = [0u8; 32];
        write_u32_be(&mut count, block as u32);

        h.reset();
        h.input(salt);
        h.input(&count);
        h.result(&mut hsalt);

        bcrypt_hash(&hpass, &hsalt, &mut out);
        let mut tmp = out;

        for _ in 1..rounds {
            h.reset();
            h.input(&tmp);
            h.result(&mut hsalt);

            bcrypt_hash(&hpass, &hsalt, &mut tmp);
            for i in 0..out.len() {
                out[i] ^= tmp[i];
            }

            for i in 0..out.len() {
                let idx = i * nblocks + (block - 1);
                if idx < output.len() {
                    output[idx] = out[i];
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use sr_std::iter::repeat;

    use bcrypt_pbkdf::{bcrypt_hash, bcrypt_pbkdf};

    #[test]
    fn test_bcrypt_hash() {
        struct Test {
            hpass: [u8; 64],
            hsalt: [u8; 64],
            out: [u8; 32],
        }

        let tests = vec![
            Test {
                hpass: [
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                hsalt: [
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                out: [
                    0x46, 0x02, 0x86, 0xe9, 0x72, 0xfa, 0x83, 0x3f, 0x8b, 0x12, 0x83, 0xad, 0x8f,
                    0xa9, 0x19, 0xfa, 0x29, 0xbd, 0xe2, 0x0e, 0x23, 0x32, 0x9e, 0x77, 0x4d, 0x84,
                    0x22, 0xba, 0xc0, 0xa7, 0x92, 0x6c,
                ],
            },
            Test {
                hpass: [
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
                    0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
                    0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26,
                    0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33,
                    0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
                ],
                hsalt: [
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                out: [
                    0xb0, 0xb2, 0x29, 0xdb, 0xc6, 0xba, 0xde, 0xf0, 0xe1, 0xda, 0x25, 0x27, 0x47,
                    0x4a, 0x8b, 0x28, 0x88, 0x8f, 0x8b, 0x06, 0x14, 0x76, 0xfe, 0x80, 0xc3, 0x22,
                    0x56, 0xe1, 0x14, 0x2d, 0xd0, 0x0d,
                ],
            },
            Test {
                hpass: [
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                hsalt: [
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
                    0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
                    0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26,
                    0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33,
                    0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
                ],
                out: [
                    0xb6, 0x2b, 0x4e, 0x36, 0x7d, 0x31, 0x57, 0xf5, 0xc3, 0x1e, 0x4d, 0x2c, 0xba,
                    0xfb, 0x29, 0x31, 0x49, 0x4d, 0x9d, 0x3b, 0xdd, 0x17, 0x1d, 0x55, 0xcf, 0x79,
                    0x9f, 0xa4, 0x41, 0x60, 0x42, 0xe2,
                ],
            },
            Test {
                hpass: [
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
                    0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
                    0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26,
                    0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33,
                    0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
                ],
                hsalt: [
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
                    0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
                    0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26,
                    0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33,
                    0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
                ],
                out: [
                    0xc6, 0xa9, 0x5f, 0xe6, 0x41, 0x31, 0x15, 0xfb, 0x57, 0xe9, 0x9f, 0x75, 0x74,
                    0x98, 0xe8, 0x5d, 0xa3, 0xc6, 0xe1, 0xdf, 0x0c, 0x3c, 0x93, 0xaa, 0x97, 0x5c,
                    0x54, 0x8a, 0x34, 0x43, 0x26, 0xf8,
                ],
            },
        ];

        for t in tests.iter() {
            let mut out = [0u8; 32];
            bcrypt_hash(&t.hpass, &t.hsalt, &mut out);
            //assert!(out == t.out);
        }
    }

    #[test]
    fn test_openbsd_vectors() {
        struct Test {
            password: Vec<u8>,
            salt: Vec<u8>,
            rounds: u32,
            out: Vec<u8>,
        }

        let tests = vec!(
            Test{
                password: b"password".to_vec(),
                salt: b"salt".to_vec(),
                rounds: 4,
                out: vec![
                    0x5b, 0xbf, 0x0c, 0xc2, 0x93, 0x58, 0x7f, 0x1c, 0x36, 0x35, 0x55, 0x5c, 0x27, 0x79, 0x65, 0x98,
                    0xd4, 0x7e, 0x57, 0x90, 0x71, 0xbf, 0x42, 0x7e, 0x9d, 0x8f, 0xbe, 0x84, 0x2a, 0xba, 0x34, 0xd9],
            }, Test{
                password: b"password".to_vec(),
                salt: vec![0],
                rounds: 4,
                out: vec![0xc1, 0x2b, 0x56, 0x62, 0x35, 0xee, 0xe0, 0x4c, 0x21, 0x25, 0x98, 0x97, 0x0a, 0x57, 0x9a, 0x67],
            }, Test{
                password: vec![0],
                salt: b"salt".to_vec(),
                rounds: 4,
                out: vec![0x60, 0x51, 0xbe, 0x18, 0xc2, 0xf4, 0xf8, 0x2c, 0xbf, 0x0e, 0xfe, 0xe5, 0x47, 0x1b, 0x4b, 0xb9],
            }, Test{
                password: b"password\x00".to_vec(),
                salt: b"salt\x00".to_vec(),
                rounds: 4,
                out: vec![
                    0x74, 0x10, 0xe4, 0x4c, 0xf4, 0xfa, 0x07, 0xbf, 0xaa, 0xc8, 0xa9, 0x28, 0xb1, 0x72, 0x7f, 0xac,
                    0x00, 0x13, 0x75, 0xe7, 0xbf, 0x73, 0x84, 0x37, 0x0f, 0x48, 0xef, 0xd1, 0x21, 0x74, 0x30, 0x50],
            }, Test{
                password: b"pass\x00wor".to_vec(),
                salt: b"sa\x00l".to_vec(),
                rounds: 4,
                out: vec![0xc2, 0xbf, 0xfd, 0x9d, 0xb3, 0x8f, 0x65, 0x69, 0xef, 0xef, 0x43, 0x72, 0xf4, 0xde, 0x83, 0xc0],
            }, Test{
                password: b"pass\x00word".to_vec(),
                salt: b"sa\x00lt".to_vec(),
                rounds: 4,
                out: vec![0x4b, 0xa4, 0xac, 0x39, 0x25, 0xc0, 0xe8, 0xd7, 0xf0, 0xcd, 0xb6, 0xbb, 0x16, 0x84, 0xa5, 0x6f],
            }, Test{
                password: b"password".to_vec(),
                salt: b"salt".to_vec(),
                rounds: 8,
                out: vec![
                    0xe1, 0x36, 0x7e, 0xc5, 0x15, 0x1a, 0x33, 0xfa, 0xac, 0x4c, 0xc1, 0xc1, 0x44, 0xcd, 0x23, 0xfa,
                    0x15, 0xd5, 0x54, 0x84, 0x93, 0xec, 0xc9, 0x9b, 0x9b, 0x5d, 0x9c, 0x0d, 0x3b, 0x27, 0xbe, 0xc7,
                    0x62, 0x27, 0xea, 0x66, 0x08, 0x8b, 0x84, 0x9b, 0x20, 0xab, 0x7a, 0xa4, 0x78, 0x01, 0x02, 0x46,
                    0xe7, 0x4b, 0xba, 0x51, 0x72, 0x3f, 0xef, 0xa9, 0xf9, 0x47, 0x4d, 0x65, 0x08, 0x84, 0x5e, 0x8d],
            }, Test{
                password: b"password".to_vec(),
                salt: b"salt".to_vec(),
                rounds: 42,
                out: vec![0x83, 0x3c, 0xf0, 0xdc, 0xf5, 0x6d, 0xb6, 0x56, 0x08, 0xe8, 0xf0, 0xdc, 0x0c, 0xe8, 0x82, 0xbd],
            }, Test{
                password: b"Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_vec(),
                salt: b"salis\x00".to_vec(),
                rounds: 8,
                out: vec![0x10, 0x97, 0x8b, 0x07, 0x25, 0x3d, 0xf5, 0x7f, 0x71, 0xa1, 0x62, 0xeb, 0x0e, 0x8a, 0xd3, 0x0a],
            }, Test{
                password: vec![0x0d, 0xb3, 0xac, 0x94, 0xb3, 0xee, 0x53, 0x28, 0x4f, 0x4a, 0x22, 0x89, 0x3b, 0x3c, 0x24, 0xae],
                salt: vec![0x3a, 0x62, 0xf0, 0xf0, 0xdb, 0xce, 0xf8, 0x23, 0xcf, 0xcc, 0x85, 0x48, 0x56, 0xea, 0x10, 0x28],
                rounds: 8,
                out: vec![0x20, 0x44, 0x38, 0x17, 0x5e, 0xee, 0x7c, 0xe1, 0x36, 0xc9, 0x1b, 0x49, 0xa6, 0x79, 0x23, 0xff],
            }, Test{
                password: vec![0x0d, 0xb3, 0xac, 0x94, 0xb3, 0xee, 0x53, 0x28, 0x4f, 0x4a, 0x22, 0x89, 0x3b, 0x3c, 0x24, 0xae],
                salt: vec![0x3a, 0x62, 0xf0, 0xf0, 0xdb, 0xce, 0xf8, 0x23, 0xcf, 0xcc, 0x85, 0x48, 0x56, 0xea, 0x10, 0x28],
                rounds: 8,
                out: vec![
                    0x20, 0x54, 0xb9, 0xff, 0xf3, 0x4e, 0x37, 0x21, 0x44, 0x03, 0x34, 0x74, 0x68, 0x28, 0xe9, 0xed,
                    0x38, 0xde, 0x4b, 0x72, 0xe0, 0xa6, 0x9a, 0xdc, 0x17, 0x0a, 0x13, 0xb5, 0xe8, 0xd6, 0x46, 0x38,
                    0x5e, 0xa4, 0x03, 0x4a, 0xe6, 0xd2, 0x66, 0x00, 0xee, 0x23, 0x32, 0xc5, 0xed, 0x40, 0xad, 0x55,
                    0x7c, 0x86, 0xe3, 0x40, 0x3f, 0xbb, 0x30, 0xe4, 0xe1, 0xdc, 0x1a, 0xe0, 0x6b, 0x99, 0xa0, 0x71,
                    0x36, 0x8f, 0x51, 0x8d, 0x2c, 0x42, 0x66, 0x51, 0xc9, 0xe7, 0xe4, 0x37, 0xfd, 0x6c, 0x91, 0x5b,
                    0x1b, 0xbf, 0xc3, 0xa4, 0xce, 0xa7, 0x14, 0x91, 0x49, 0x0e, 0xa7, 0xaf, 0xb7, 0xdd, 0x02, 0x90,
                    0xa6, 0x78, 0xa4, 0xf4, 0x41, 0x12, 0x8d, 0xb1, 0x79, 0x2e, 0xab, 0x27, 0x76, 0xb2, 0x1e, 0xb4,
                    0x23, 0x8e, 0x07, 0x15, 0xad, 0xd4, 0x12, 0x7d, 0xff, 0x44, 0xe4, 0xb3, 0xe4, 0xcc, 0x4c, 0x4f,
                    0x99, 0x70, 0x08, 0x3f, 0x3f, 0x74, 0xbd, 0x69, 0x88, 0x73, 0xfd, 0xf6, 0x48, 0x84, 0x4f, 0x75,
                    0xc9, 0xbf, 0x7f, 0x9e, 0x0c, 0x4d, 0x9e, 0x5d, 0x89, 0xa7, 0x78, 0x39, 0x97, 0x49, 0x29, 0x66,
                    0x61, 0x67, 0x07, 0x61, 0x1c, 0xb9, 0x01, 0xde, 0x31, 0xa1, 0x97, 0x26, 0xb6, 0xe0, 0x8c, 0x3a,
                    0x80, 0x01, 0x66, 0x1f, 0x2d, 0x5c, 0x9d, 0xcc, 0x33, 0xb4, 0xaa, 0x07, 0x2f, 0x90, 0xdd, 0x0b,
                    0x3f, 0x54, 0x8d, 0x5e, 0xeb, 0xa4, 0x21, 0x13, 0x97, 0xe2, 0xfb, 0x06, 0x2e, 0x52, 0x6e, 0x1d,
                    0x68, 0xf4, 0x6a, 0x4c, 0xe2, 0x56, 0x18, 0x5b, 0x4b, 0xad, 0xc2, 0x68, 0x5f, 0xbe, 0x78, 0xe1,
                    0xc7, 0x65, 0x7b, 0x59, 0xf8, 0x3a, 0xb9, 0xab, 0x80, 0xcf, 0x93, 0x18, 0xd6, 0xad, 0xd1, 0xf5,
                    0x93, 0x3f, 0x12, 0xd6, 0xf3, 0x61, 0x82, 0xc8, 0xe8, 0x11, 0x5f, 0x68, 0x03, 0x0a, 0x12, 0x44],
            },
        );

        for t in tests.iter() {
            let mut out: Vec<u8> = repeat(0).take(t.out.len()).collect();
            bcrypt_pbkdf(&t.password[..], &t.salt[..], t.rounds, &mut out[..]);
            assert_eq!(out, t.out);
        }
    }
}

#[cfg(all(test, feature = "with-bench"))]
mod bench {
    use bcrypt_pbkdf::bcrypt_pbkdf;
    use test::Bencher;

    #[bench]
    fn bench_bcrypt_pbkdf_5_32(b: &mut Bencher) {
        let pass = [0u8; 16];
        let salt = [0u8; 16];
        let mut out = [0u8; 32];

        b.iter(|| {
            bcrypt_pbkdf(&pass, &salt, 5, &mut out);
        });
    }
}
