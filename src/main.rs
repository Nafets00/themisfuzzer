use std::collections::HashMap;

use bytes::Bytes;
use rand::{rngs::ThreadRng, Rng};
use themis_patch_core::{app::{Request, Response}, net::{Message, Raw}, protocol::ProtocolTag};
use themis_patch_core::net::NetworkMessage;
use themis_patch_core::comms::*;
use themis_patch_pbft::{messages::*, requests::RequestEntryPatch, test::{setup_patch_pbft, setup_patch_pbft_backup, PBFTPatchContext}, ViewState};
use themis_pbft::requests::RequestEntry;
use themisfuzzer::context::*;
use themisfuzzer::patch::generate_pre_prepare;
use futures_util::{poll, FutureExt, StreamExt};
use std::collections;



async fn getfunc(sequence:u64, view:u64, request:Bytes) -> Raw<ProtocolTag<PBFTTag>>{

    let res = PrePrepare::new(sequence, view, request).pack();
    match res {
        Ok(message) => {
            
            message
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }

}

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




    print!("pbft: {:?} \n\n\n", normal);
    print!("patch_pbft: {:?} \n", patched);
    false
}


#[tokio::main]
async fn main() {
    let rng = ThreadRng::default();
    let normal_pbft = setup_pbft();
    let mut backup = setup_patch_pbft_backup();
    let mut pbft = setup_patch_pbft();

    //compare_versions(&normal_pbft, &pbft);
    
    

    println!("pbft: {:?}\n\n", pbft.pbft);
    println!("pbft_backup: {:?}\n\n", backup.pbft);
    
    

    for i in 0..2{
        for j in 0..2{
            
            //let mut buf = [0u8; 16]; // 16-byte array
            //rand::thread_rng().fill(&mut buf);
            //let slice: &[u8] = &buf;
            //let byte = Bytes::copy_from_slice(slice);
            //println!("slisc{:?}", slice);
            //let msg = getfunc(j+i, backup.pbft.view(), byte).await;
            //let _ = backup.pbft.comms.replicas.send(Message::new(1,0, msg)).await;
            //let _ = generate_pre_prepare(slice, &mut pbft.pbft,&mut backup.pbft,  i+j, i, j).await;
            
        }
        
    }
    println!("pbft: {:?}\n\n", pbft.pbft);
    println!("pbft_backup: {:?}\n\n", backup.pbft);
    

    
}
