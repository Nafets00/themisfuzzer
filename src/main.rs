use std::{collections::HashMap, task::Poll};

use bytes::Bytes;
use rand::{rngs::ThreadRng, Rng};
use themis_patch_core::{app::{Request, Response}, net::{Message, Raw}, protocol::ProtocolTag};
use themis_patch_core::net::NetworkMessage;
use themis_patch_core::comms::*;
use themis_patch_pbft::{messages::*, requests::RequestEntryPatch, test::{setup_patch_pbft, setup_patch_pbft_backup, PBFTPatchContext}, ViewState};
use themis_pbft::requests::RequestEntry;
use themisfuzzer::{comp::{self, compare_versions}, context::{self, *}, patch::generate_pre_prepare};

use futures_util::{future::poll_fn, poll, FutureExt, Stream, StreamExt};
use std::collections;




async fn getfunc(sequence:u64, view:u64, request:Bytes) -> Raw<ProtocolTag<PBFTTag>>{

    let res = Commit::new(sequence, view, request).pack();
    match res {
        Ok(message) => {
            println!("message generation success");
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
    let mut patch = themis_patch_pbft::test::setup_with_checkpoint_patch(false, 1000);
    let mut pbft = context::setup_with_checkpoint(false, 1000);

    
    let mut rng = ThreadRng::default();

    let mut buffer = [0u8; 16]; 
    let bytes  = Bytes::new();

    rng.fill(&mut buffer); 
    println!("emp: {}", &buffer.is_empty());
    let buf: &[u8] = &buffer;
    let bytes = Bytes::new();
    let bytes2 =Bytes::new();
    let bytes3 =Bytes::new();
    let bytes4 =Bytes::new();
    let bytes5 = Bytes::new();
    let bytes6 =Bytes::new();



    let msg_patch = themis_patch_pbft::messages::PrePrepare::new(1, 0, bytes);
    let msg_preprep_patch = themis_patch_core::net::Message::new(0,0, msg_patch);
    let res_patch = msg_preprep_patch.pack();
    match res_patch{
        Ok(message) =>{let _ = patch.pbft.on_message(message).await;}
        Err(e) =>{println!("err")}
    }

    let msg = themis_pbft::messages::PrePrepare::new(1, 0, bytes2);
    let msg_preprep = themis_core::net::Message::new(0, 0, msg);
    let res = msg_preprep.pack();
    match res{
        Ok(message) =>{let _ = pbft.pbft.on_message(message).await;}
        Err(e) =>{println!("err")}
    }


    let msg = themis_patch_pbft::messages::Prepare::new(1, 0, bytes3);
    let msg_prep_patch = themis_patch_core::net::Message::new(0,0, msg);
    let res_patch = msg_prep_patch.pack();
    match res_patch{
        Ok(message) =>{let _ = patch.pbft.on_message(message).await;}
        Err(e) =>{println!("err")}
    }

    let msg = themis_pbft::messages::Prepare::new(1, 0, bytes4);
    let msg_prep = themis_core::net::Message::new(0, 0, msg);
    let res = msg_prep.pack();
    match res{
        Ok(message) =>{let _ = pbft.pbft.on_message(message).await;}
        Err(e) =>{println!("err")}
    }



    let msg_patch = themis_patch_pbft::messages::Commit::new(1, 0, bytes5);
    let res_patch = themis_patch_core::net::Message::new(0,0, msg_patch).pack();
    match res_patch{
        Ok(message) =>{let _ = patch.pbft.on_message(message).await;}
        Err(e) =>{println!("err")}
    }

    let msg = themis_pbft::messages::Commit::new(1, 0, bytes6);
    let res = themis_core::net::Message::new(0, 0, msg).pack();
    match res{
        Ok(message) =>{let _ = pbft.pbft.on_message(message).await;}
        Err(e) =>{println!("err")}
    }



    let checkpoint_proof = Vec::new().into();
    let msg_box = Box::new([msg_prep_patch]);
    let pp = themis_patch_pbft::messages::PrepareProof(msg_preprep_patch, msg_box);
    let box_pp = Box::new([pp]);
    let msg = themis_patch_pbft::messages::ViewChange::new(1, 0, checkpoint_proof, box_pp);
    let res = themis_patch_core::net::Message::new(0, 0, msg).pack();
    match res{
        Ok(message) =>{let _ = patch.pbft.on_message(message).await;}
        Err(e) =>{println!("err")}
    }

    let checkpoint_proof = Vec::new().into();
    let msg_box = Box::new([msg_prep]);
    let pp = themis_pbft::messages::PrepareProof(msg_preprep, msg_box);
    let box_pp = Box::new([pp]);
    let msg_patch = themis_pbft::messages::ViewChange::new(1, 0, checkpoint_proof, box_pp);
    let res_patch = themis_core::net::Message::new(0,0, msg_patch).pack();
    match res_patch{
        Ok(message) =>{let _ = pbft.pbft.on_message(message).await;}
        Err(e) =>{println!("err")}
    }

    compare_versions(&pbft, &patch, buf, 1, 1, 1, 1); 

    println!("pbft: {:?}\n\n", pbft.pbft);
    println!("patch: {:?}\n\n", patch.pbft);




    //compare_versions(&pbft, &patch, buf);
    println!("view{}", pbft.pbft.view());
    println!("pbft: {:?}\n\n", pbft.pbft.state);
    println!("patch: {:?}\n\n", patch.pbft.state);
    
    
}
