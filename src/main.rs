use bytes::Bytes;
use rand::{rngs::ThreadRng, Rng};
use themis_patch_core::{app::{Request, Response}, net::{Message, Raw}, protocol::ProtocolTag};
use themis_patch_core::net::NetworkMessage;
use themis_patch_core::comms::*;
use themis_patch_pbft::{messages::*, test::{setup_patch_pbft, setup_patch_pbft_backup}};
use themisfuzzer::patch::generate_pre_prepare;
use futures_util::{poll, FutureExt, StreamExt};



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

#[tokio::main]
async fn main() {
    let rng = ThreadRng::default();
    let mut backup = setup_patch_pbft_backup();
    let mut pbft = setup_patch_pbft();

    

    //println!("pbft: {:?}", pbft.pbft);
    println!("backup:{:?}", backup.pbft);
    println!("");
    println!("");
    println!("");
    print!("pbft: {:?}", pbft.pbft);
    println!("");
    println!("");
    println!("");
    
    

    for i in 0..2{
        for j in 0..2{
            
            let mut buf = [0u8; 16]; // 16-byte array
            rand::thread_rng().fill(&mut buf);
            let slice: &[u8] = &buf;
            let byte = Bytes::copy_from_slice(slice);
            println!("slisc{:?}", slice);
            let msg = getfunc(j+i, backup.pbft.view(), byte).await;
            let _ = backup.pbft.comms.replicas.send(Message::new(1,0, msg)).await;
            //let _ = generate_pre_prepare(slice, &mut pbft.pbft,&mut backup.pbft,  i+j, i, j).await;
            
        }
        
    }

    println!("pbft: {:?}", backup.pbft);
    println!("");
    println!("");
    println!("");
    print!("pbft: {:?}", pbft);

    

}
