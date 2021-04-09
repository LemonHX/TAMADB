use bytes::Bytes;
use crossbeam_epoch::Atomic;
use smallvec::SmallVec;
use std::{borrow::Cow, cmp::min, convert::TryInto, intrinsics::cttz, mem::MaybeUninit, sync::{atomic::AtomicBool, Arc}};
use std::{
    sync::atomic::{AtomicU64, Ordering::*},
    thread::sleep_ms,
};
#[derive(Clone)]
// I decide to fuck NULL char
enum NodeInnerType {
    // well just a leaf
    // delete will not remove the data instead set deleted to true
    Leaf {
        deleted: bool,
        key: Bytes,
        value: Bytes,
    },
    // search through keys and get index
    Node4 {
        key: [u8; 4],
        child_pointers: [Option<NodeRef>; 4],
    },
    // search through
    // x86_48 -> SIMD look through bit mask
    Node16 {
        key: [u8; 16],
        child_pointers: [Option<NodeRef>; 16],
    },
    // two array index look up
    Node48 {
        key: [u8; 256],
        child_pointers: [Option<NodeRef>; 48],
    },
    // direct index
    Node256 {
        child_pointers: [Option<NodeRef>; 256],
    },
}
impl NodeInnerType {
    #[inline(always)]
    fn capacity(&self) -> u16 {
        match self {
            NodeInnerType::Node4 { .. } => 4,
            NodeInnerType::Node16 { .. } => 16,
            NodeInnerType::Node48 { .. } => 48,
            NodeInnerType::Node256 { .. } => 256,
            _ => panic!("???"),
        }
    }
}

// atomic copy on write pointer to Node
struct NodeRef{
    version: AtomicU64,
    pointer: Atomic<Node>,
}
impl Clone for NodeRef{
    fn clone(&self) -> Self {
        // should not be null in any time
        let cloned_data = unsafe{
            // acquire
            self.pointer.load_consume(&crossbeam_epoch::pin())
                .as_ref()
                .unwrap()
                .clone()
            };
        Self{
            version: AtomicU64::new(self.version.load(SeqCst) + 2),
            pointer: Atomic::new(cloned_data)
        }
    }
}
impl NodeRef{
    // merge in changes to original node
    // Ok -> safely merged in
    // Err -> failed to merge in due node is newer than cloned
    fn merge_in(&self,other:Self) -> Result<(),()>{
        let guard = &crossbeam_epoch::pin();
        // other only has one holder so should could use Relaxed memory order
        let other_version = other.version.load(Relaxed);
        let other_pointer = other.pointer.load(Relaxed,guard);

        self.version.compare_exchange(
            other_version - 2, 
            other_version, 
            SeqCst, 
            SeqCst).map_err(|_| ())?;
        self.pointer.compare_exchange(
            self.pointer.load_consume(guard),
            other.pointer.load(Relaxed,guard),
            SeqCst, 
            SeqCst,
            guard
        ).map_err(|_| ())?;
        Ok(())
    }
}

#[derive(Clone)]
struct Node {
    len: u16,
    // normally we uses pessimistic way to insert
    // when prefix is long enough smallvec couldn't hold on stack
    // then we use optimistic approch.
    // we use .spilled() to check which method we will use

    // complete prefix node key = prefix + key[i]
    // leaf should has empty prefix
    prefix: SmallVec<[u8; 8]>,
    inner: NodeInnerType,
}

impl Node {
    fn get_common_prefix(&self, key: &Bytes, depth: usize) -> usize {
        let max_comp = min(self.prefix.len(), key.len() - depth);
        let mut idx = 0;
        while idx < max_comp {
            if self.prefix[idx] != key[idx] {
                return idx;
            } else {
                idx += 1;
            }
        }
        return idx;
    }

    fn find_child(&self, kl: u8) -> Option<NodeRef> {
        debug_assert!(kl != 0x00);
        let guard = crossbeam_epoch::pin();
        match &self.inner {
            NodeInnerType::Node4 {
                key,
                child_pointers,
            } => {
                for i in 0..self.len as usize {
                    if key[i] == kl {
                        return *child_pointers.get(i).unwrap();
                    }
                }
                return None;
            }
            NodeInnerType::Node16 {
                key,
                child_pointers,
            } => {
                let mut bf = 0;
                let mask = (1 << self.len) - 1;

                #[cfg(all(
                    any(target_arch = "x86", target_arch = "x86_64"),
                    target_feature = "sse2"
                ))]
                unsafe {
                    use core::arch::x86_64::{
                        _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
                    };
                    let keysc = _mm_set1_epi8(kl as i8);
                    let cmptor = _mm_loadu_si128(key.as_ptr() as *mut _);
                    let cmp = _mm_cmpeq_epi8(keysc, cmptor);

                    bf = _mm_movemask_epi8(cmp) & mask;
                }
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                {
                    for i in 0..16 {
                        if key[i] == kl {
                            bf |= (1 << i);
                        }
                    }
                    bf &= mask;
                }
                if bf != 0 {
                    return *child_pointers.get(cttz(bf) as usize).unwrap();
                } else {
                    None
                }
            }
            NodeInnerType::Node48 {
                key,
                child_pointers,
            } => {
                let i = key[kl as usize];
                if i != 0 {
                    return *child_pointers.get((i - 1) as usize).unwrap();
                } else {
                    None
                }
            }
            NodeInnerType::Node256 { child_pointers } => *child_pointers.get(kl as usize)?,
            NodeInnerType::Leaf {
                key,
                value,
                deleted,
            } => None,
        }
    }
    
    fn insert_to_current_node(&mut self, kl: u8, value: Bytes) -> Option<()> {
        // if does not contains enough space
        // if contains enough space
        match self.inner{
            NodeInnerType::Leaf { deleted, key, value } => {panic!("Fuck me")}
            NodeInnerType::Node4 { key, child_pointers } => {
                key[self.len as usize] = kl;
                child_pointers[self.len as usize] = None;
            }
            NodeInnerType::Node16 { key, child_pointers } => {
                key[self.len as usize] = kl;
                child_pointers[self.len as usize] = None;
            }
            NodeInnerType::Node48 { key, child_pointers } => {
                key[kl as usize] = self.len as u8 - 1;
                child_pointers[self.len as usize - 1] = None;
            }
            NodeInnerType::Node256 { child_pointers } => {
                // None -> Leaf
                child_pointers[kl as usize] = None;
            }
        };
        todo!()
    }

    
}

fn search(node: &Node, key: Bytes, depth: usize) -> Option<(Bytes, Bytes)> {
    let mut node = node;
    let guard = &crossbeam_epoch::pin();
    while true{
        let prefix_len = node.get_common_prefix(&key, depth);
        if prefix_len != node.prefix.len() {
            return None;
        } else {
            let depth = depth + prefix_len;
            //TODO: search the usage of guard
            //TODO: due to atomic prob the pointer could be the
            // old pointer before expation so the search result is not 'correct'
            // in other words "dirty read" will be happend
            unsafe {
                match node.find_child(key[depth]){
                    Some(n) => {
                        node = n.pointer.load_consume(guard).as_ref().unwrap();
                    }
                    None => {
                        return None;
                    }
                }
            }
        }
    }
    todo!()
}

#[test]
fn test_find_child() {}
