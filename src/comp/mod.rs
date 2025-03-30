use std::collections::HashMap;

use bytes::Bytes;
use themis_patch_core::{app::{Request, Response}, net::{Message, Raw}, protocol::ProtocolTag};
use themis_patch_pbft::{messages::*, requests::RequestEntryPatch, test::{setup_patch_pbft, setup_patch_pbft_backup, PBFTPatchContext}, ViewState};
use themis_pbft::requests::RequestEntry;
use crate::context::*;




fn compare_state(normal:&themis_pbft::ViewState , patched:&ViewState)->bool{
    if (normal.is_regular() && patched.is_regular()) || (normal.is_view_change() && patched.is_view_change()){
        return true;
    }
    false
}

fn compare_request(normal: &HashMap<Bytes,RequestEntry>, patched:&HashMap<Bytes,RequestEntryPatch>)->bool{
    if  normal.len() == patched.len(){
         for req in normal.values(){
            return false;
         }
    }

    false
}

pub fn compare_versions(pbft: &PBFTContext, patch_pbft: &PBFTPatchContext) -> bool{

    let normal = &pbft.pbft;
    let patched = &patch_pbft.pbft;
    
    if normal.id() != patched.id() {
        println!("different ids")
    }
    if normal.low_mark() != patched.low_mark() {
        //println!("normal: {}, patch: {}", normal.low_mark(), patched.low_mark());
        println!("diff low mark")
    }
    if normal.next_sequence() != patched.next_sequence(){
        println!("diff seq")
    }
    if normal.last_commit != patched.last_commit {
        println!("diff commits")
    }
    if normal.view() != patched.view() {
        println!("diff view")
    }
    if !compare_state(&normal.state, &patched.state) {
        println!("diff states")
    }
    if !compare_request(&normal.requests.requests, &patched.requests.requests){
        println!("diff Request store")
    }
    
    false
}