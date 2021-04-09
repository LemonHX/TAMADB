use core::panic;

use smallvec::SmallVec;

use super::{ARTNodeDispatched, ARTNodeHeader, ChildType};
pub struct Node256 {
    header: ARTNodeHeader,
    children: [Option<ChildType>; 256],
}
impl ARTNodeDispatched for Node256 {
    fn insert(&mut self, k: u8, n: ChildType) {
        self.children[k as usize] = Some(n);
        self.header.len += 1;
    }

    fn remove(&mut self, k: u8) {
        self.children[k as usize] = None;
        self.header.len -= 1;
    }

    fn change(&mut self, k: u8, n: ChildType) -> bool {
        self.children[k as usize] = Some(n);
        true
    }

    fn clone_to(&self, dst: &mut dyn ARTNodeDispatched) {
        for i in 0..256 {
            if let Some(n) = &self.children[i] {
                dst.insert(i as u8, n.clone());
            }
        }
    }

    fn get_child(&self, k: u8) -> Option<ChildType> {
        self.children[k as usize].clone()
    }

    fn get_second_child(&self, k: u8) {
        panic!("FUCK!")
    }

    fn get_any_child(&self) -> ChildType {
        let mut ind = 0;
        for i in 0..256 {
            if let Some(n) = &self.children[i] {
                match n {
                    n @ ChildType::NodePtr(_) => {
                        ind = i;
                    }
                    l @ ChildType::Leaf { .. } => {
                        return l.clone();
                    }
                }
            }
        }
        return self.children[ind].clone().unwrap();
    }

    fn get_children(&self, k_start: u8, k_end: u8) -> smallvec::SmallVec<[(u8, ChildType); 256]> {
        loop{
            let mut need_restart = false;
            let version = self.header.read_lock_or_restart(&need_restart);
            if need_restart{
                continue;
            }
            let mut res = SmallVec::new();
            for i in 0..256{
                if let Some(n) = &self.children[i]{
                    res.push((i as u8,n.clone()));
                }
            }
            self.header.read_unlock_or_restart(version, &need_restart);
            if need_restart {
                continue;
            }
            return res;
        } 
    }

    fn delete_children(&mut self) {
        todo!()
    }

    fn is_full(&self) -> bool {
        false
    }

    fn is_under_full(&self) -> bool {
        self.header.len < 38
    }
}
