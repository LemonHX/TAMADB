#![feature(core_intrinsics)]
use std::sync::atomic::AtomicPtr;

use crossbeam_epoch::Atomic;
use smallvec::SmallVec;

// simple command line interface of TAMADB
fn main() {
    // well just testing SIMD
    use std::intrinsics::cttz;
    let mut keys:[u8;16] = [1 ^ 128,2 ^ 128,4 ^ 128,5 ^ 128,6 ^ 128,7 ^ 128,8 ^ 128,9 ^ 128,10 ^ 128,11 ^ 128,12 ^ 128,13 ^ 128,14 ^ 128,15 ^ 128,16 ^ 128,0];
    let key = 3;
    let kf = key ^ 128;
            unsafe {
        use core::arch::x86_64::{
            _mm_cmplt_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
        };
        let cmp = _mm_cmplt_epi8(
            _mm_set1_epi8(kf as i8),
            _mm_loadu_si128(keys.as_mut_ptr() as *mut _),
        );
        let bf = _mm_movemask_epi8(cmp) & (0xFFFF >> (16 - 15));
        let pos = if bf != 0 { cttz(bf) } else { 15 } as usize;
        let a = 0;
    }
    let mut pos = 0;
    for i in 0..15 as usize {
        if (keys[i] as i8) < (kf as i8) {
            pos += 1;
        }
    }
    let b = 0;
}
