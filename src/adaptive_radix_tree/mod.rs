use std::sync::atomic::AtomicU64;

use bytes::Bytes;
use crossbeam_epoch::Atomic;
use enum_dispatch::*;
mod node4;
use node4::Node4;
mod node16;
use node16::Node16;
mod node48;
use node48::Node48;
mod node256;
use node256::Node256;
use smallvec::SmallVec;
mod utils;
use utils::*;
type Prefix = SmallVec<[u8;11]>;
#[derive(Clone)]
enum ChildType{
    NodePtr(Atomic<Node>),
    // i guess key is implicit right?
    // but just in case I also added cuz bytes is just a ptr
    Leaf{key: Bytes, value: Bytes},
}
#[repr(u8)]
enum NodeType{
    Node4 = 0,
    Node16,
    Node48,
    Node256,
}
impl From<u8> for NodeType{
    #[inline(always)]
    fn from(u: u8) -> Self {
        debug_assert!(u < 4);
        unsafe{std::mem::transmute(u)}
    }
}


struct ARTNodeHeader{
    // 2b type 60b version 1b lock 1b outdate mark
    type_version_lock_outdate_flag: AtomicU64,
    // prefix of current node
    perfix: Prefix,
    // len of current node
    len: u16,
    // capacity of current node
    capacity: u16,
}


impl ARTNodeHeader{
    #[inline(always)]
    fn get_type(&self) -> NodeType{
        NodeType::from((self.type_version_lock_outdate_flag.load(std::sync::atomic::Ordering::SeqCst) >> 62) as u8)
    }

    // only use it once !!!!
    #[inline(always)]
    fn set_type(&self,ty:NodeType){
        self.type_version_lock_outdate_flag.fetch_add(convert_to_flag(ty), std::sync::atomic::Ordering::SeqCst);
    }

    #[inline(always)]
    fn len(&self) -> usize{
        unimplemented!()
    }

    fn write_lock_or_restart(&self,need_restart: &mut bool){
        let mut version = self.read_lock_or_restart(need_restart);
        if *need_restart {
            return
        }else{
            self.upgrade_to_lock_or_restart(&mut version, need_restart);
            if *need_restart {
                return
            }
        }
    }
    fn upgrade_to_lock_or_restart(&self, version: &mut u64, need_restart: &mut bool){
        match self.type_version_lock_outdate_flag.compare_exchange(*version, *version + 0b10, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst){
            Ok(_) => {
                *version += 0b10;
            }
            Err(_) => {
                *need_restart = true;
            }
        }
    }
    fn write_unlock(&self){
        self.type_version_lock_outdate_flag.fetch_add(0b10, std::sync::atomic::Ordering::SeqCst);
    }
    fn read_lock_or_restart(&self, need_restart: &bool) -> u64{
        unimplemented!()
    }
    //TODO: the impl == readUnlockOrRestart
    fn check_or_restart(){}
    fn read_unlock_or_restart(&self, start_read: u64, need_restart: &bool){}

    
    /// call when node is lock
    fn write_unlock_outdate(){}
    fn has_prefix(){}
    fn get_prefix(){}
    fn set_prefix(){}
    fn add_prefix_befor(){}
    fn get_prefix_len(){}
}

// is not unsafe! look! I did lock it 23333

#[enum_dispatch]
trait ARTNodeDispatched{
    // for 4 16 48 will be a sort but performace lose could be ignored
    // for 256 without sort due to only 256 possible outcomes
    fn insert(&mut self, k:u8, n: ChildType);
    // TODO: just write a flag instead of removing
    fn remove(&mut self, k:u8);
    fn change(&mut self, k:u8, n: ChildType) -> bool;
    // will call insert 114514 times
    fn clone_to(&self, dst: &mut dyn ARTNodeDispatched);

    fn get_child(&self, k:u8) -> Option<ChildType>;
    fn get_second_child(&self, k:u8);
    fn get_any_child(&self) -> ChildType;
    fn get_children(&self, k_start: u8, k_end:u8) -> SmallVec<[(u8,ChildType);256]>;
    
    // TODO: ?
    fn delete_children(&mut self);

    fn is_full(&self) -> bool;
    fn is_under_full(&self) -> bool;
}

#[enum_dispatch(ARTNodeDispatched)]
enum Node{
    Node4,
    Node16,
    Node48,
    Node256,
}




#[inline(always)]
fn convert_to_flag(ty:NodeType) -> u64{
    let v:u64 = ty as u8 as u64;
    v << 62
}