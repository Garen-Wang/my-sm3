static IV: [u32; 8] = [
    0x7380166f, 0x4914b2b9, 0x172442d7, 0xda8a0600, 0xa96f30bc, 0x163138aa, 0xe38dee4d, 0xb0fb0e4e,
];

fn t(j: usize) -> u32 {
    assert!(j <= 63);
    if j <= 15 {
        0x79cc4519
    } else {
        0x7a879d8a
    }
}

fn ff1(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

fn ff2(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (x & z) | (y & z)
}

fn gg1(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

fn gg2(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}

fn p0(x: u32) -> u32 {
    x ^ x.rotate_left(9) ^ x.rotate_left(17)
}

fn p1(x: u32) -> u32 {
    x ^ x.rotate_left(15) ^ x.rotate_left(23)
}

fn update_w(x: &mut [u32; 132], j: usize) {
    let p1_param = x[j - 16] ^ x[j - 9] ^ x[j - 3].rotate_left(15);
    let temp = p1(p1_param) ^ x[j - 13].rotate_left(7) ^ x[j - 6];
    // let p1_param = w(x, j - 16) ^ w(x, j - 9) ^ w(x, j - 3).rotate_left(15);
    // let temp = p1(p1_param) ^ w(x, j - 13).rotate_left(7) ^ w(x, j - 6);
    x[j] = temp;
}

fn update_w2(x: &mut [u32; 132], j: usize) {
    x[j + 68] = x[j] ^ x[j + 4];
}

pub fn get_padding_message(message: &[u8]) -> Vec<u8> {
    let len = message.len() * 8;
    let mut a = Vec::from(message);
    let mut k = if (len + 1) % 512 <= 448 { 448 - (len + 1) % 512 + 1 } else { 448 + 512 - (len + 1) % 512 + 1 };
    // println!("At bit level, len = {}, k = {}", len, k);
    // println!("At byte level, len = {}, k = {}", len / 8, k / 4);
    assert_eq!(k % 4, 0);
    k /= 8;
    if k >= 1 {
        a.push(0b1000_0000u8);
        k -= 1;
    }
    if k >= 1 {
        a.append(&mut vec![0u8; k]);
    }
    let c: [u8; 8] = (len as u64).to_be_bytes();
    a.append(&mut Vec::from(c));
    a
}

pub fn compress(padding_message: &[u8]) -> [u32; 8] {
    let n_iterations = padding_message.len() / 64;
    let mut state = IV;
    for i in 0..n_iterations {
        let mut new_w = [0u32; 132];
        for j in 0..16 {
            new_w[j] = u32::from_be_bytes([
                padding_message[i*64 + j*4], padding_message[i*64 + j*4+1], padding_message[i*64 + j*4+2], padding_message[i*64 + j*4+3],
            ]);
        }
        for j in 16..=67 {
            update_w(&mut new_w, j);
        }
        for j in 0..64 {
            update_w2(&mut new_w, j);
        }
        // println!("{:x?}", new_w);
        // println!("len of new_w = {}", new_w.len());

        // compress

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];
        for j in 0..64 {
            let ss1 = (a.rotate_left(12).wrapping_add(e).wrapping_add(t(j).rotate_left(j as u32))).rotate_left(7);
            let ss2 = ss1 ^ a.rotate_left(12);
            let ff = if j <= 15 { ff1 } else { ff2 };
            let gg = if j <= 15 { gg1 } else { gg2 };
            let tt1 = ff(a, b, c).wrapping_add(d).wrapping_add(ss2).wrapping_add(new_w[j + 68]);
            let tt2 = gg(e, f, g).wrapping_add(h).wrapping_add(ss1).wrapping_add(new_w[j]);
            d = c;
            c = b.rotate_left(9);
            b = a;
            a = tt1;
            h = g;
            g = f.rotate_left(19);
            f = e;
            e = p0(tt2);
        }
        state[0] ^= a;
        state[1] ^= b;
        state[2] ^= c;
        state[3] ^= d;
        state[4] ^= e;
        state[5] ^= f;
        state[6] ^= g;
        state[7] ^= h;           
    }
    // println!("res = {:x?}", state);
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    // only available for something less than 448 bits
    #[test]
    fn sm3_example1() {
        let message_str = "abc";
        let message = message_str.as_bytes();
        let len = message.len() * 8;
        let mut a = Vec::from(message);
        let mut k = if (len + 1) % 512 <= 448 { 448 - (len + 1) % 512 + 1 } else { 448 + 512 - (len + 1) % 512 + 1 };
        // println!("At bit level, len = {}, k = {}", len, k);
        // println!("At byte level, len = {}, k = {}", len / 8, k / 4);
        assert_eq!(k % 4, 0);
        k /= 8;
        if k >= 1 {
            a.push(0b1000_0000u8);
            k -= 1;
        }
        if k >= 1 {
            a.append(&mut vec![0u8; k]);
        }
        let c: [u8; 8] = (len as u64).to_be_bytes();
        a.append(&mut Vec::from(c));
        // println!("{:x?}", a);
        // println!("len of a = {}", a.len());

        // expand message

        let mut new_w = [0u32; 132];
        for j in 0..16 {
            new_w[j] = u32::from_be_bytes([
                a[j * 4], a[j * 4 + 1], a[j * 4 + 2], a[j * 4 + 3],
            ]);
        }
        for j in 16..=67 {
            update_w(&mut new_w, j);
        }
        for j in 0..64 {
            update_w2(&mut new_w, j);
        }
        // println!("{:x?}", new_w);
        // println!("len of new_w = {}", new_w.len());

        // compress

        let mut state: [u32; 8] = IV;
        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];
        for j in 0..64 {
            let ss1 = (a.rotate_left(12).wrapping_add(e).wrapping_add(t(j).rotate_left(j as u32))).rotate_left(7);
            let ss2 = ss1 ^ a.rotate_left(12);
            let ff = if j <= 15 { ff1 } else { ff2 };
            let gg = if j <= 15 { gg1 } else { gg2 };
            let tt1 = ff(a, b, c).wrapping_add(d).wrapping_add(ss2).wrapping_add(new_w[j + 68]);
            let tt2 = gg(e, f, g).wrapping_add(h).wrapping_add(ss1).wrapping_add(new_w[j]);
            d = c;
            c = b.rotate_left(9);
            b = a;
            a = tt1;
            h = g;
            g = f.rotate_left(19);
            f = e;
            e = p0(tt2);
        }
        state[0] ^= a;
        state[1] ^= b;
        state[2] ^= c;
        state[3] ^= d;
        state[4] ^= e;
        state[5] ^= f;
        state[6] ^= g;
        state[7] ^= h;           

        println!("res = {:x?}", state);
    }

    #[test]
    fn sm3_example2() {
        let message_str = "abcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd";
        let message = message_str.as_bytes();
        let len = message.len() * 8;
        let mut a = Vec::from(message);
        let mut k = if (len + 1) % 512 <= 448 { 448 - (len + 1) % 512 + 1 } else { 448 + 512 - (len + 1) % 512 + 1 };
        // println!("At bit level, len = {}, k = {}", len, k);
        // println!("At byte level, len = {}, k = {}", len / 8, k / 4);
        assert_eq!(k % 4, 0);
        k /= 8;
        if k >= 1 {
            a.push(0b1000_0000u8);
            k -= 1;
        }
        if k >= 1 {
            a.append(&mut vec![0u8; k]);
        }
        let c: [u8; 8] = (len as u64).to_be_bytes();
        a.append(&mut Vec::from(c));
        // println!("{:x?}", a);
        // println!("len of a = {}", a.len());

        // expand message

        let n_iterations = a.len() / 64;
        let mut state = IV;
        for i in 0..n_iterations {
            let mut new_w = [0u32; 132];
            for j in 0..16 {
                new_w[j] = u32::from_be_bytes([
                    a[i*64 + j*4], a[i*64 + j*4+1], a[i*64 + j*4+2], a[i*64 + j*4+3],
                ]);
            }
            for j in 16..=67 {
                update_w(&mut new_w, j);
            }
            for j in 0..64 {
                update_w2(&mut new_w, j);
            }
            // println!("{:x?}", new_w);
            // println!("len of new_w = {}", new_w.len());

            // compress

            let mut a = state[0];
            let mut b = state[1];
            let mut c = state[2];
            let mut d = state[3];
            let mut e = state[4];
            let mut f = state[5];
            let mut g = state[6];
            let mut h = state[7];
            for j in 0..64 {
                let ss1 = (a.rotate_left(12).wrapping_add(e).wrapping_add(t(j).rotate_left(j as u32))).rotate_left(7);
                let ss2 = ss1 ^ a.rotate_left(12);
                let ff = if j <= 15 { ff1 } else { ff2 };
                let gg = if j <= 15 { gg1 } else { gg2 };
                let tt1 = ff(a, b, c).wrapping_add(d).wrapping_add(ss2).wrapping_add(new_w[j + 68]);
                let tt2 = gg(e, f, g).wrapping_add(h).wrapping_add(ss1).wrapping_add(new_w[j]);
                d = c;
                c = b.rotate_left(9);
                b = a;
                a = tt1;
                h = g;
                g = f.rotate_left(19);
                f = e;
                e = p0(tt2);
            }
            state[0] ^= a;
            state[1] ^= b;
            state[2] ^= c;
            state[3] ^= d;
            state[4] ^= e;
            state[5] ^= f;
            state[6] ^= g;
            state[7] ^= h;           

            println!("res = {:x?}", state);
        }
    }
}

pub struct Sm3;

impl Sm3 {
    pub fn digest(message: &[u8]) -> [u32; 8] {
        let padding_message = get_padding_message(message);
        compress(&padding_message)
    }
}