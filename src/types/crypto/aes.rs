use log::{debug, trace, warn};

use ::crypto::buffer::{BufferResult, RefWriteBuffer, RefReadBuffer};

use crate::types::*;
use crate::types::util::{pack_u64, unpack_64_bit};

pub fn decrypt_with_key(key: &Vec<u8>, data: &Vec<u8>) -> Vec<u8> {
    const ZERO_IV : &[u8] = &[0u8; 16];
    let mut dec = ::crypto::aes::ecb_decryptor(
        ::crypto::aes::KeySize::KeySize256,
        key.as_slice(),
        ::crypto::blockmodes::NoPadding
    );
    trace!("unwrapping key - setup output");
    let mut out : Vec<u8> = vec![0u8; data.len()];
    let mut output = RefWriteBuffer::new(out.as_mut_slice());
    let mut input = RefReadBuffer::new(data.as_slice());
    // debug!("unwrapping key - run dec");
    let result = dec.decrypt(&mut input, &mut output, true);
    trace!("is_Err: {}", result.is_err());
    trace!("res: {}", hex::encode(&out));
    return out
}

pub fn unwrap_key(kek: &[u8], wpky: &Vec<u8>) -> Vec<u8> {
    trace!("Key: {:x?}", kek);
    trace!("Wrapped: {:x?}", wpky);

    trace!("unwrapping key!");
    let mut c : Vec<u64> = vec![];

    for i in 0..(wpky.len()/8) {
        let slice : &[u8] = &wpky.as_slice()[i*8..i*8+8];
        let val = unpack_64_bit(&slice);

        if let Some(val) = val {
            c.push(u64::from_be_bytes(val));
        } else {
            panic!("Couldn't unwrap 64 bit value from provided wrapper.");
        }
    }

    trace!("C: {:x?}", c);

    let n = c.len() - 1;

    trace!("N: {:x?}", n);
    let mut r : Vec<u64> = vec![0; n+1];

    trace!("R: {:x?}", r);
    let mut a = c[0];

    trace!("A: {:x?}", a);

//         # Copy C into R, after the first value.
// for i in xrange(1,n+1):
//     R[i] = C[i]
    // Copy c into r
    for i in 1..(n+1) {
        r[i] = c[i]
    }

    trace!("key sz: {}", kek.len());
    trace!("c: {:?}", c);
    trace!("n: {:?}", n);
    trace!("r: {:?}", r);
    trace!("a: {:?}", a);

    for j in (0..6).rev() {
        for i in (1..n+1).rev() {
            trace!("unwrapping key - it a={} n={} j={} i={}", a, n, j, i);
            let val = (a as u64) ^ ((n as u64)*(j as u64)+(i as u64));
            trace!("a component: {:x?}", val);
            trace!("r[i={}] component: {:x?}", i, r[i]);
            let mut packed = val.to_be_bytes().to_vec();
            trace!("packed component (a): {:x?}", packed);
            let packed2 = r[i].to_be_bytes();
            packed.extend_from_slice(&packed2);
            trace!("packed component: {:x?}", packed);
            // debug!("unwrapping key - make ecb");
            /// TODO: data param is not matching python!!!!!!!!

            trace!("key_length: {}", kek.len() * 8);
            trace!("decrypt(cipher=aes_{}_ecb, kek={}, iv={}, data={}", kek.len() * 8, hex::encode(kek), hex::encode(ZERO_IV), hex::encode(packed.as_slice()));
            const ZERO_IV : &[u8] = &[0u8; 16];
            // let cipher = match kek.len() * 8 {
            //     128 => Cipher::aes_128_ecb(),
            //     256 => Cipher::aes_256_ecb(),
            //     _ => panic!("unsupported input key length: {}", kek.len() * 8)
            // };
            
            // let out = decrypt(cipher, kek, Some(ZERO_IV), packed.as_slice());
            // match out {
            //     Ok(out) => {
            //         // let mut dec = crypto::aes::ecb_decryptor(crypto::aes::KeySize::KeySize256, kek, crypto::blockmodes::DecPadding {
            //         //     padding: crypto::blockmodes::PkcsPadding 
            //         // });
            //         // debug!("unwrapping key - setup output");
            //         // let mut out : Vec<u8> = vec![0u8; 16];
            //         // let mut output = crypto::buffer::RefWriteBuffer::new(out.as_mut_slice());
            //         // let mut input = crypto::buffer::RefReadBuffer::new(packed.as_slice());
            //         // // debug!("unwrapping key - run dec");
            //         // let result = dec.decrypt(&mut input, &mut output, true);
            //         a = u64::from_be_bytes(KeyBag::unpack_64_bit(&out.as_slice()[0..8]).unwrap());
            //         r[i] = u64::from_be_bytes(KeyBag::unpack_64_bit(&out.as_slice()[8..16]).unwrap());

            //         debug!("res a: {}", a);
            //         debug!("res r[i]: {}", r[i]);
            //         debug!("out: {:?}", out);
            //     },
            //     Err(res) => panic!("decrypt err: {}", res)
            // }
            {
                // use crypto::symmetriccipher::BlockDecryptor;
                // let dec = crypto::aessafe::AesSafe256Decryptor::new(kek);
                // let mut out : Vec<u8> = vec![0u8; 16];
                // dec.decrypt_block(&packed, &mut out.as_mut_slice());
                
                trace!("aes_ecb_256_dec({:x?})", kek);
                let mut dec = ::crypto::aes::ecb_decryptor(::crypto::aes::KeySize::KeySize256, kek, ::crypto::blockmodes::NoPadding);
                trace!("unwrapping key - setup output");
                let mut out : Vec<u8> = vec![0u8; 16];
                let mut output = RefWriteBuffer::new(out.as_mut_slice());
                let mut input = RefReadBuffer::new(packed.as_slice());
                // debug!("unwrapping key - run dec");
                let result = dec.decrypt(&mut input, &mut output, true);
                trace!("is_Err: {}", result.is_err());
                trace!("res: {}", hex::encode(&out));
                
                a = u64::from_be_bytes(unpack_64_bit(&out.as_slice()[0..8]).unwrap());
                trace!("new a: {:x?}", a);
                r[i] = u64::from_be_bytes(unpack_64_bit(&out.as_slice()[8..16]).unwrap());
                trace!("new r[{}]: {:x?}", i, a);
            }
        }
    }

    if a != 0xa6a6a6a6a6a6a6a6 {
        warn!("got iv: 0x{:x} {}", a, a);
        panic!("unexpected resulant iv");
    }

    let mut result : Vec<u8> = Vec::new();
    trace!("result vector {:x?}", r);
    for i in 1..r.len() {
        // let packed =  Vec::new();
        // let other : Vec<u8> = packed..into();
        let packed = pack_u64(r[i]);
        trace!("r[i={}] = {:x} == {:x?}", i, r[i], &packed);
        
        result.extend_from_slice(&packed);
    }

    trace!("decrypt result: {}", hex::encode(&result));

    return result
}
