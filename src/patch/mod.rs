use bytes::Bytes;

use rand::prelude::*;
use themis_patch_core::{app::{Request, Response}, net::Message, protocol::Proposal};
use themis_patch_pbft::messages::*;
use themis_patch_pbft::test::PBFTPatchContext;



pub async fn generate_pre_prepare(buf: &[u8], pbft: &mut themis_patch_pbft::PBFT, replica:&mut themis_patch_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> () {
    

    let view = pbft.view();
    let bytes = Bytes::copy_from_slice(buf);
    let msg = PrePrepare::new(sequence, view, bytes);
    let res = Message::new(source,destination, msg).pack();
    let cop = Message::new(source, destination,PrePrepare::new(sequence,view,Bytes::copy_from_slice(buf))).pack();
    
   
    
    match res {
        Ok(message) => {
            
            let _ = pbft.on_message(message).await;
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }

    match cop {
        Ok(message) => {
            let _ = replica.on_message(message).await;
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
    
}

async fn generate_prepare(buf: &[u8], pbft: &mut themis_patch_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> () {

    let view = pbft.view();

    let request = Bytes::copy_from_slice(buf);
    let msg = Prepare::new(sequence,view, request);
    let res = Message::new(source, destination, msg).pack();
    match res {
        Ok(message) => {
           let _ = pbft.on_message(message).await;
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}

async fn generate_commit(buf: &[u8], pbft: &mut themis_patch_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> () {

    let view = pbft.view();

    let request = Bytes::copy_from_slice(buf);
    let msg = Commit::new(sequence, view, request);
    let res = Message::new(source, destination, msg).pack();
    match res {
        Ok(message) => {
           let _ = pbft.on_message(message).await;
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}


async fn generate_view_change(buf: &[u8], pbft: &mut themis_patch_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> () {

    let mut rng = ThreadRng::default();
    
    let new_view = pbft.view()+1;
    let checkpoint = rng.r#gen();
    let source = pbft.id();

    let checkpoint_proof = Vec::new().into();
    let prepares= Vec::new().into();
    let res = Message::new(source, destination,ViewChange::new(new_view, checkpoint, checkpoint_proof, prepares)).pack();
    
    match res {
        Ok(message) => {
           let _ = pbft.on_message(message).await;
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}
fn generate_nonraw_view_change(buf: &[u8], sequence:u64, curr_view:u64) -> Message<ViewChange>{

    let mut rng = ThreadRng::default();

    let new_view = curr_view+1;
    let checkpoint = rng.r#gen();
    let checkpoint_proof = Vec::new().into();
    let prepares= Vec::new().into();
    let res = Message::broadcast(0,ViewChange::new(new_view, checkpoint, checkpoint_proof, prepares));
    res
}


async fn generate_new_view(buf: &[u8], pbft: &mut themis_patch_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> () {


    let view = pbft.view();
    let view_changes = Box::new([generate_nonraw_view_change(buf, sequence, view)]);
    let pre_prepares  = Vec::new().into();
    let res = Message::new(source, destination,NewView::new(view, view_changes, pre_prepares)).pack();
    match res {
        Ok(message) => {
           let _ = pbft.on_message(message).await;
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}


async fn generate_assign(buf: &[u8], pbft: &mut themis_patch_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> () {

    let mut rng = ThreadRng::default();

    let batch_size = rng.gen_range(1..10);
    let batch: Vec<Message<Request>> = (0..batch_size)
        .map(|_| generate_random_request_noraw(buf, sequence, source, destination))
        .collect();
    let proposal = Proposal::try_from(batch).expect("Proposal cannot be empty");
    let res = Message::new(source, destination,Assign::new(sequence, proposal)).pack();
    match res {
        Ok(message) => {
           let _ = pbft.on_message(message).await;
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
    
}



async fn generate_full_checkpoint(buf: &[u8], pbft: &mut themis_patch_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> () {

    let mut rng = ThreadRng::default();

    let handle = Bytes::from(vec![rng.r#gen(); 64]);
    let data = Bytes::copy_from_slice(buf);
    
    let res = Message::new(source, destination,FullCheckpoint::new(sequence, handle, data)).pack();
    match res {
        Ok(message) => {
           let _ = pbft.on_message(message).await;
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}

async fn generate_get_checkpoint(buf: &[u8], pbft: &mut themis_patch_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> () {

    

    let res = Message::new(source, destination,GetCheckpoint::new(sequence)).pack();
    match res {
        Ok(message) => {
           let _ = pbft.on_message(message).await;
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}

async fn generate_forward(buf: &[u8], pbft: &mut themis_patch_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> () {

    let payload = Forward(Proposal::default());
    let res = Message::new(source, destination, payload).pack();
    match res {
        Ok(message) => {
           let _ = pbft.on_message(message).await;
        }
        Err(e) => {
            eprintln!("Cannot pack message: {:?}", e); 
            panic!("Error packing message: {:?}", e);
        }
    }
}


fn generate_random_request_noraw(buf: &[u8], sequence:u64, source:u64, destination:u64) -> Message<Request> {

    let payload: Bytes = Bytes::copy_from_slice(buf);
    let request = Request::new(sequence, payload);

    Message::new(source, destination, request)
}

fn generate_random_request(buf: &[u8], pbft: &mut themis_patch_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> () {

    let payload: Bytes = Bytes::copy_from_slice(buf);
    let request = Request::new(sequence, payload);
    let message = Message::new(source, destination,request);

    pbft.requests.add_request(message, Some(sequence));
}


async fn generate_random_response(buf: &[u8], pbft: &mut themis_patch_pbft::PBFT, sequence:u64, source:u64, destination:u64) -> () {

    let mut rng = ThreadRng::default();

    let payload: Bytes = Bytes::copy_from_slice(buf);
    let contact: Option<u64> = if rng.gen_bool(0.5) { Some(rng.r#gen()) } else { None };
    let response = Response::with_contact(sequence, payload, contact.unwrap_or_default());
    
    let res = Message::new(source, destination, response);
    let _ = pbft.on_response(res).await;
}



pub async fn to_fuzz_patch(buf: &[u8], sequence:u64, pbft_context: &mut PBFTPatchContext, replica_context: &mut PBFTPatchContext, source:u64, destination:u64){
    let mut rng = ThreadRng::default();
    
 
    let rndvar: u8 = rng.gen_range(0..11);
    
    match rndvar {
        0 => generate_pre_prepare(buf, &mut pbft_context.pbft, &mut replica_context.pbft,sequence, source, destination,).await,
        1 => generate_assign(buf, &mut pbft_context.pbft, sequence, source, destination).await,
        2 => generate_commit(buf, &mut pbft_context.pbft, sequence, source, destination).await,
        3 => generate_forward(buf,&mut pbft_context.pbft, sequence, source, destination).await,
        4 => generate_full_checkpoint(buf,&mut pbft_context.pbft, sequence, source, destination).await,
        5 => generate_get_checkpoint(buf,&mut pbft_context.pbft, sequence, source, destination).await,
        6 => generate_new_view(buf,&mut pbft_context.pbft, sequence, source, destination).await,
        7 => generate_prepare(buf,&mut pbft_context.pbft, sequence, source, destination).await,
        8 => generate_random_request(buf, &mut pbft_context.pbft, sequence, source, destination),
        9 => generate_random_response(buf,&mut pbft_context.pbft, sequence, source, destination).await,
        10 => generate_view_change(buf,&mut pbft_context.pbft, sequence, source, destination).await,
        _ => println!("state of pbft: {:?}", pbft_context.pbft)
    };
}