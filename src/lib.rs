use std::thread::Thread;

use bytes::Bytes;
use context::{setup_pbft, PBFTContext};
use rand::{distributions::DistString, prelude::*};
use serde::de;
use themis_core::{app::{request, Request, Response}, net::{Message, Raw, Sequenced}, protocol::{Proposal, ProtocolTag}};
use themis_pbft::messages::*;
use futures_util::{poll, FutureExt, Stream, StreamExt};
pub mod context;
pub mod patch;
pub mod comp;
pub mod restarter;
pub mod logger;




async fn generate_pre_prepare(buf: &[u8], view:u64, pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> bool {    
  
    let mut bytes = Bytes::copy_from_slice(buf);
    if buf.len() <= 1{
        bytes = Bytes::new();
    }
    let msg = PrePrepare::new(sequence, view, bytes);
    let res = Message::new(source, destination, msg).pack();
    
    match res {
        Ok(message) => {
           let _ = pbft.comms.replicas.send(message).await;
           true
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
    
}

async fn generate_prepare(buf: &[u8], view:u64,pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> bool {


    let mut request = Bytes::copy_from_slice(buf);
    if buf.len() <= 1{
        request = Bytes::new();
    }
    let msg = Prepare::new(sequence,view, request);
    let res = Message::new(source, destination, msg).pack();

    match res {
        Ok(message) => {
            let _ = pbft.comms.replicas.send(message).await;
            true
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}

async fn generate_commit(buf: &[u8],view:u64, pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> bool {

    let mut request = Bytes::copy_from_slice(buf);
    if buf.len() <= 1{
        request = Bytes::new();
    }
    let msg = Commit::new(sequence, view, request);
    let res = Message::new(source, destination, msg).pack();
    match res {
        Ok(message) => {
            let _ = pbft.comms.replicas.send(message).await;
            true
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}


async fn generate_view_change(buf: &[u8], view:u64,pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> bool {

    let new_view = view+1;

    let checkpoint_proof = Vec::new().into();
    let prepares= Vec::new().into();
    let res = Message::new(source, destination,ViewChange::new(new_view, 0, checkpoint_proof, prepares)).pack();
    
    match res {
        Ok(message) => {
           let _ = pbft.comms.replicas.send(message).await;
           true
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}
fn generate_nonraw_view_change(buf: &[u8], view:u64,sequence:u64, curr_view:u64, source:u64, destination:u64) -> Message<ViewChange>{
    let new_view = view+1;
    let checkpoint_proof = Vec::new().into();
    let prepares= Vec::new().into();
    let res = Message::new(source, destination,ViewChange::new(new_view, 0, checkpoint_proof, prepares));
    res
}


async fn generate_new_view(buf: &[u8],view:u64,pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> bool {

    
    let view_changes = Box::new([generate_nonraw_view_change(buf,view, sequence, view, source, destination)]);

    let pre_prepares  = Vec::new().into();

    let res = Message::new(source, destination,NewView::new(view, view_changes, pre_prepares)).pack();
    match res {
        Ok(message) => {
           let _ = pbft.comms.replicas.send(message).await;
           true
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}


async fn generate_assign(buf: &[u8], view:u64,pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> bool {
    
    
    let batch_size = buf.len();
    let batch: Vec<Message<Request>> = (0..batch_size)
        .map(|_| generate_random_request_noraw(buf, view, sequence, source, destination))
        .collect();
    let proposal = Proposal::try_from(batch).expect("Proposal cannot be empty");
    let res = Message::new(source, destination,Assign::new(sequence, proposal)).pack();
    match res {
        Ok(message) => {
           let _ = pbft.comms.replicas.send(message).await;
           true
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
    
}

async fn generate_checkpoint(buf: &[u8], view:u64,pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64)-> bool{
    let mut bytes = Bytes::copy_from_slice(buf);
    if buf.len() <= 1{
        bytes = Bytes::new();
    }
    let check = Checkpoint::new(sequence, bytes);
    let res = Message::new(source, destination, check).pack();
    match res {
        Ok(message) => {
           let _ = pbft.comms.replicas.send(message).await;
           return true
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}

async fn generate_full_checkpoint(buf: &[u8], view:u64,pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> bool {

    let (l, r) = buf.split_at(buf.len()/2);

    let handle =  Bytes::copy_from_slice(l);
    let data = Bytes::copy_from_slice(r);
    
    let res = Message::new(source, destination,FullCheckpoint::new(sequence, handle, data)).pack();
    match res {
        Ok(message) => {
           let _ = pbft.comms.replicas.send(message).await;
           true
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}

async fn generate_get_checkpoint(buf: &[u8], view:u64,pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> bool {

    let res = Message::new(source, destination,GetCheckpoint::new(sequence)).pack();
    match res {
        Ok(message) => {
           let _ = pbft.comms.replicas.send(message).await;
           true
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}

async fn generate_forward(buf: &[u8],view:u64, pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> bool {


    let payload = Forward(Proposal::default());
    let res = Message::new(source, destination, payload).pack();
    match res {
        Ok(message) => {
           let _ = pbft.comms.replicas.send(message).await;
           true
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}


fn generate_random_request_noraw(buf: &[u8], view:u64,sequence:u64, source:u64, destination:u64) -> Message<Request> {

    let payload: Bytes = Bytes::copy_from_slice(buf);
    let request = Request::new(sequence, payload);

    Message::new(source, destination, request)
}

async fn generate_random_request(buf: &[u8],view:u64,pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> bool {

    let digest: Bytes = Bytes::copy_from_slice(buf);

    let exec = pbft.requests.execute_requests(&digest, &mut pbft.comms.app).await;
    match exec{
        Ok(_)=> {return true}
        Err(_) => {return false}
    }
}


async fn generate_random_response(buf: &[u8], view:u64,pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> bool {

    let mut contact :Option<u64>= Some((0));
    let payload: Bytes = Bytes::copy_from_slice(buf);

    if buf.len() <= 10{
        contact = None;
    }
    else {
        contact = Some(buf[0].into());
    }


    let response = Response::with_contact(sequence, payload, contact.unwrap_or_default());
    
    let res = Message::new(source, destination, response);
    let _ = pbft.on_response(res).await;
    true
}
async fn generate_correct_seq(buf: &[u8], view:u64,pbft: &mut themis_pbft::PBFT, sequence:u64, source:u64, destination:u64)->bool{
    
    let bytes2 = Bytes::copy_from_slice(buf);
    let bytes4 = Bytes::copy_from_slice(buf);
    let bytes6 = Bytes::copy_from_slice(buf);



    let msg = themis_pbft::messages::PrePrepare::new(sequence, view, bytes2);
    let msg_preprep = themis_core::net::Message::new(source, destination, msg);
    let res = msg_preprep.pack();
    match res{
        Ok(message) =>{let _ = pbft.on_message(message).await;}
        Err(e) =>{println!("err")}
    }

    let msg = themis_pbft::messages::Prepare::new(sequence, view ,bytes4);
    let msg_prep = themis_core::net::Message::new(source, destination, msg);
    let res = msg_prep.pack();
    match res{
        Ok(message) =>{let _ = pbft.on_message(message).await;}
        Err(e) =>{println!("err")}
    }


    let msg = themis_pbft::messages::Commit::new(sequence, view, bytes6);
    let res = themis_core::net::Message::new(source, destination, msg).pack();
    match res{
        Ok(message) =>{let _ = pbft.on_message(message).await;}
        Err(e) =>{println!("err")}
    }

    let checkpoint_proof = Vec::new().into();
    let msg_box = Box::new([msg_prep]);
    let pp = themis_pbft::messages::PrepareProof(msg_preprep, msg_box);
    let box_pp = Box::new([pp]);
    let msg_patch = themis_pbft::messages::ViewChange::new(view+1, 0, checkpoint_proof, box_pp);
    let res_patch = themis_core::net::Message::new(source,destination, msg_patch).pack();
    match res_patch{
        Ok(message) =>{let _ = pbft.on_message(message).await;
        true}
        Err(e) =>{println!("err"); false}
    }

}



pub async fn to_fuzz(rnd_var: u64, buf: &[u8], view:u64,sequence:u64, pbft_context: &mut PBFTContext, pbft_context2: &mut PBFTContext, source:u64, destination:u64)->bool{
    //println!("rndvar: {}", rnd_var);
    let mut ret = false;
    match rnd_var {
        0 => ret = generate_pre_prepare(buf, view,&mut pbft_context.pbft, sequence, source, destination).await,
        1 => ret = generate_assign(buf, view,&mut pbft_context.pbft, sequence, source, destination).await,
        2 => ret = generate_commit(buf, view,&mut pbft_context.pbft, sequence, source, destination).await,
        3 => ret = generate_forward(buf, view,&mut pbft_context.pbft, sequence, source, destination).await,
        4 => ret = generate_full_checkpoint(buf, view,&mut pbft_context.pbft, sequence, source, destination).await,
        5 => ret=  generate_get_checkpoint(buf, view,&mut pbft_context.pbft, sequence, source, destination).await,
        6 => ret = generate_new_view(buf,view, &mut pbft_context.pbft, sequence, source, destination).await,
        7 => ret= generate_prepare(buf, view,&mut pbft_context.pbft, sequence, source, destination).await,
        8 => ret = generate_random_response(buf, view,&mut pbft_context.pbft, sequence, source, destination).await,
        9 => ret = generate_view_change(buf, view,&mut pbft_context.pbft, sequence, source, destination).await,
        10 => ret = generate_correct_seq(buf, view, &mut pbft_context.pbft, sequence, source, destination).await,
        11 => ret = generate_random_request(buf, view, &mut pbft_context.pbft, sequence, source, destination).await,
        12 => ret = generate_checkpoint(buf, view, &mut pbft_context.pbft, sequence, source, destination).await,
        _ => {}
    };

    let next_msg = pbft_context.r.next().now_or_never();
            match next_msg {
                Some(m) => {
                    match m {
                        Some(message) =>{
                            let _ = pbft_context.pbft.on_message(message).await;
                        }
                        None => {}
                    }
                    
                }
                None => {
                    
                }
            }
    
    let next_msg = pbft_context2.r.next().now_or_never();
    match next_msg {
        Some(m) => {
            match m {
                Some(message) =>{
                    let _ = pbft_context2.pbft.on_message(message).await;
                }
                None => {}
            }
            
        }
        None => {
            
        }
    }
    return ret;
    //println!("{:?}", pbft_context.pbft);
}