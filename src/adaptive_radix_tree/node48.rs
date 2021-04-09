use smallvec::SmallVec;

use super::{ARTNodeDispatched, ARTNodeHeader, ChildType};

const EMPTY_INDEX : u8 = 48;

pub struct Node48{
    header: ARTNodeHeader,
    // empty == 48
    index: [u8;258],
    children: [Option<ChildType>;48],
}
impl ARTNodeDispatched for Node48{
    fn insert(&mut self, k:u8, n: ChildType) {
        let mut pos = self.header.len as usize;
        // case after deletion
        if let Some(_) = self.children[pos]{
            for i in 0..48{
                if let None = self.children[i]{
                    pos = i;
                }
            }
        }
        self.children[pos] = Some(n);
        self.index[k as usize] = pos as u8;
        self.header.len += 1;
    }

    fn remove(&mut self, k:u8) {
        debug_assert!(self.index[k as usize] != EMPTY_INDEX);
        self.children[self.index[k as usize] as usize] = None;
        self.index[k as usize] = EMPTY_INDEX;
        self.header.len -= 1;
    }

    fn change(&mut self, k:u8, n: ChildType) -> bool {
        self.children[self.index[k as usize] as usize] = Some(n);
        return true;
    }

    fn clone_to(&self, dst: &mut dyn ARTNodeDispatched) {
        for i in 0..256{
            if self.index[i] != EMPTY_INDEX{
                dst.insert(i as u8, self.children[self.index[i as usize] as usize].clone().unwrap())
            }
        }
    }

    fn get_child(&self, k:u8) -> Option<ChildType> {
        if self.index[k as usize] != EMPTY_INDEX{
            self.children[self.index[k as usize] as usize].clone()
        }else{
            None
        }
    }

    fn get_second_child(&self, k:u8) {
        todo!()
    }

    fn get_any_child(&self) -> ChildType {
        let mut ind = 0;
        for i in 0 .. 256{
            if self.index[i] != EMPTY_INDEX{
                match self.children[self.index[i] as usize].as_ref().unwrap() {
                    n@ChildType::NodePtr(_) => {
                        ind = i;
                    }
                    l@ChildType::Leaf { .. } => {
                        return l.clone();
                    }
                }
            }
        }
        return self.children[self.index[ind] as usize].clone().unwrap();
    }

    fn get_children(&self, k_start: u8, k_end:u8) -> smallvec::SmallVec<[(u8,ChildType);256]> {
        loop{
            let mut need_restart = false;
            let version = self.header.read_lock_or_restart(&need_restart);
            if need_restart{
                continue;
            }

            let mut res = SmallVec::new();
            for i in k_start..=k_end{
                let index = self.index[i as usize];
                if index != EMPTY_INDEX{
                    res.push((i, self.children[index as usize].clone().unwrap()));
                }
            }

            // unlock and check version
            self.header.read_unlock_or_restart(version,&need_restart);
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
        self.header.len == 48
    }

    fn is_under_full(&self) -> bool {
        self.header.len == 12
    }
}