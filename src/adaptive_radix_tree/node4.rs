use smallvec::SmallVec;

use super::{ARTNodeDispatched, ARTNodeHeader, ChildType};

pub struct Node4{
    header: ARTNodeHeader,
    kv: [(u8,ChildType);4]
}

impl ARTNodeDispatched for Node4{
    fn insert(&mut self, k:u8, n: super::ChildType) {
        // unsafe version
        unsafe{ // aha!
            let mut pos = 0;
            for i in 0..self.header.len as usize{
                if self.kv[i].0 < k{
                    pos += 1;
                }
            }
            let kvptr = self.kv.as_mut_ptr();
            std::ptr::copy(kvptr.add(pos), kvptr.add(pos + 1), self.header.len() - pos);
            self.kv[pos] = (k,n);
            self.header.len += 1;
        }
    }

    fn remove(&mut self, k:u8) {
        // unsafe version
        unsafe{
            for i in 0..self.header.len as usize{
                if self.kv[i].0 == k{
                    let kvptr = self.kv.as_mut_ptr();
                    std::ptr::copy(kvptr.add(i+1), kvptr.add(i), self.header.len as usize - i - 1);
                    self.header.len -= 1;
                    return;
                }
            }
        }
    }

    fn change(&mut self, k:u8, n: super::ChildType) -> bool{
        for i in 0..self.header.len as usize{
            if self.kv[i].0 == k{
                self.kv[i].1 = n;
                return true;
            }
        }
        false
    }

    fn clone_to(&self, dst: &mut dyn ARTNodeDispatched) {
        for i in 0..self.header.len as usize{
            dst.insert(self.kv[i].0, self.kv[i].1.clone());
        }
    }

    fn get_child(&self, k:u8) -> Option<super::ChildType> {
        for i in 0..self.header.len as usize{
            if self.kv[i].0 == k{
                return Some(self.kv[i].1.clone());
            }
        }
        None
    }

    fn get_second_child(&self, k:u8) {
        todo!()
    }
    // TODO: redesign
    fn get_any_child(&self) -> super::ChildType {
        for i in 0..self.header.len as usize{
            match self.kv[i].1.clone(){
                l@ChildType::Leaf { .. } => {
                    return l;
                }
                _ => {}
            }
        }
        return self.kv[self.header.len as usize].1.clone();
    }
    // TODO: retry X times
    fn get_children(&self, k_start: u8, k_end:u8) -> smallvec::SmallVec<[(u8,super::ChildType);256]> {
        loop{
            let mut need_restart = false;
            // lock and get version
            let v = self.header.read_lock_or_restart(&need_restart);
            if need_restart{
                continue;
            }
            let mut res = SmallVec::new();
            for i in 0..self.header.len as usize{
                res.push(self.kv[i].clone());
            }
            // unlock and check version
            self.header.read_unlock_or_restart(v,&need_restart);
            if need_restart{
                continue;
            }
            // everything good
            return res;
        }
    }

    fn delete_children(&mut self) {
        todo!()
    }

    fn is_full(&self) -> bool {
        self.header.len == 4
    }

    fn is_under_full(&self) -> bool {
        false
    }
}