use std::{collections::HashMap, task::Poll};

use bytes::Bytes;
use rand::{rngs::ThreadRng, Rng};
use themis_patch_core::{app::{Request, Response}, net::{Message, Raw}, protocol::ProtocolTag};
use themis_patch_core::net::NetworkMessage;
use themis_patch_core::comms::*;
use themis_patch_pbft::{messages::*, requests::RequestEntryPatch, test::{setup_patch_pbft, setup_patch_pbft_backup, PBFTPatchContext}, ViewState};
use themis_pbft::requests::RequestEntry;
use themisfuzzer::{context::*, patch::generate_pre_prepare};

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
    let rng = ThreadRng::default();
    let sequence = 1;
    let view = 0;
    let mut backup = setup_patch_pbft_backup();
    let mut pbft = setup_patch_pbft();
    
  
            let mut buf = [0u8; 16]; // 16-byte array
            rand::thread_rng().fill(&mut buf);
            let slice: &[u8] = &buf;
         
            let byte = Bytes::copy_from_slice(slice);
         
            let msg = getfunc(sequence, view, byte).await;
            

            let send = Message::new(1, 0, msg);
           
            
            let res2 = pbft.pbft.comms.replicas.send(send).await;

            let next_msg = pbft.r.next().now_or_never();
            match next_msg {
                Some(m) => {
                    match m {
                        Some(message) =>{
                            let _ = pbft.pbft.on_message(message).await;
                        }
                        None => {println!("empty")}
                    }
                    
                }
                None => {
                    println!("emty1");
                }
            }

            
            let next_msg2 = backup.r.next().now_or_never();
            match next_msg2 {
            Some(m) => {
                match m {
                    Some(message) =>{
                        let _ = backup.pbft.on_message(message).await;
                    }
                    None => {println!("empty")}
                }
                
            }
            None => {
                println!("emty2");
            }
        }
            

    println!("pbft: {:?}\n\n", pbft.pbft);
    println!("pbft_backup: {:?}\n\n", backup.pbft);
    

    
}
