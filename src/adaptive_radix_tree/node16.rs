use super::{ARTNodeDispatched, ARTNodeHeader, ChildType};
use std::intrinsics::cttz;
pub struct Node16 {
    header: ARTNodeHeader,
    // careful! this is not the original u8
    // is u8 key fliped signed bit for SSE
    keys: [u8; 16],
    children: [ChildType; 16],
}
impl Node16 {
    fn get_child_by_pos(&self, k: u8) -> Option<&ChildType> {
        let kf = k ^ 128;
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2"
        ))]
        unsafe {
            use core::arch::x86_64::{
                _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
            };
            let cmp = _mm_cmpeq_epi8(
                _mm_set1_epi8(kf as i8),
                _mm_loadu_si128(self.keys.as_ptr() as *mut _),
            );
            let bf = _mm_movemask_epi8(cmp) & ((1 << self.header.len) - 1);
            return self.children.get(cttz(bf) as usize);
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            for i in 0..self.header.len as usize {
                if (self.keys[i] as i8) == (kf as i8) {
                    return self.children.get(i);
                }
            }
            None
        }
    }
    fn get_child_by_pos_mut(&mut self, k: u8) -> Option<&mut ChildType> {
        let kf = k ^ 128;
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2"
        ))]
        unsafe {
            use core::arch::x86_64::{
                _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
            };
            let cmp = _mm_cmpeq_epi8(
                _mm_set1_epi8(kf as i8),
                _mm_loadu_si128(self.keys.as_mut_ptr() as *mut _),
            );
            let bf = _mm_movemask_epi8(cmp) & ((1 << self.header.len) - 1);
            return self.children.get_mut(cttz(bf) as usize);
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            for i in 0..self.header.len as usize {
                if (self.keys[i] as i8) == (kf as i8) {
                    return self.children.get_mut(i);
                }
            }
            None
        }
    }
}
impl ARTNodeDispatched for Node16 {
    fn insert(&mut self, k: u8, n: super::ChildType) {
        let mut pos = 0;
        let kf = k ^ 128;
        // get pos to insert
        {
            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "sse2"
            ))]
            unsafe {
                use core::arch::x86_64::{
                    _mm_cmplt_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
                };
                let cmp = _mm_cmplt_epi8(
                    _mm_set1_epi8(kf as i8),
                    _mm_loadu_si128(self.keys.as_mut_ptr() as *mut _),
                );
                let bf = _mm_movemask_epi8(cmp) & (0xFFFF >> (16 - self.header.len));
                pos = if bf != 0 {
                    cttz(bf) as u16
                } else {
                    self.header.len
                } as usize;
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                pos = 0;
                for i in 0..self.header.len as usize {
                    if (self.keys[i] as i8) < (kf as i8) {
                        pos += 1;
                    }
                }
            }
        }

        // insert to pos
        unsafe {
            let kptr = self.keys.as_mut_ptr();
            let cptr = self.children.as_mut_ptr();
            std::ptr::copy(kptr.add(pos), kptr.add(pos + 1), self.header.len() - pos);
            std::ptr::copy(cptr.add(pos), cptr.add(pos + 1), self.header.len() - pos);
            self.keys[pos] = kf;
            self.children[pos] = n;
            self.header.len += 1;
        }
    }

    fn remove(&mut self, k: u8) {
        todo!()
    }

    fn change(&mut self, k: u8, n: super::ChildType) -> bool {
        match self.get_child_by_pos_mut(k){
            Some(c) => {
                *c = n;
                true
            }
            None => {
                false
            }
        }
    }

    fn clone_to(&self, dst: &mut dyn ARTNodeDispatched) {
        for i in 0..self.header.len as usize {
            dst.insert(self.keys[i] ^ 128, self.children[i].clone());
        }
    }

    fn get_child(&self, k: u8) -> Option<super::ChildType> {
        self.get_child_by_pos(k).map(|r| r.clone())
    }

    fn get_second_child(&self, k: u8) {
        panic!("FUCK!")
    }

    fn get_any_child(&self) -> super::ChildType {
        for i in 0..self.header.len as usize{
            if let l@ChildType::Leaf { .. } = &self.children[i]{
                    return l.clone();
            }
        }
        return self.children[self.header.len as usize].clone();
    }

    fn get_children(
        &self,
        k_start: u8,
        k_end: u8,
    ) -> smallvec::SmallVec<[(u8, super::ChildType); 256]> {
        todo!()
    }

    fn delete_children(&mut self) {
        todo!()
    }

    fn is_full(&self) -> bool {
        self.header.len == 16
    }

    fn is_under_full(&self) -> bool {
        self.header.len < 4
    }
}
