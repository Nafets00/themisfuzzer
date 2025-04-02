use std::{collections::HashMap, process::exit};

use bytes::Bytes;
use themis_patch_pbft::{requests::RequestEntryPatch, test::PBFTPatchContext, ViewState};
use themis_pbft::requests::RequestEntry;
use crate::context::*;
use crate::logger::log_on_comp;


//works slightly different then in the thesis as it will kill the fuzzer if a differance has been found instead of resetting the state logging is done via print

fn compare_state(normal:&themis_pbft::ViewState , patched:&ViewState)->bool{
    if (normal.is_regular() && patched.is_regular()) || (normal.is_view_change() && patched.is_view_change()){
        return true;
    }
    false
}

fn compare_request(normal: &HashMap<Bytes,RequestEntry>, patched:&HashMap<Bytes,RequestEntryPatch>)->bool{

    let normal_values: Vec<_> = normal.values().collect();
    let patched_values: Vec<_> = patched.values().collect();

    
    if  normal.len() == patched.len(){
        'outer: for i in 0..normal_values.len(){
           for j in 0..patched_values.len(){
            if (normal_values[i].sequence == patched_values[j].sequence) && (normal_values[i].request.inner.payload == patched_values[j].request.inner.payload) {
                continue 'outer;
                }
            }
            return false;

        }
        return true;
    }
false
}
fn compare_request_batch(normal: &HashMap<Bytes,themis_pbft::Batch>, patched:&HashMap<Bytes,themis_patch_pbft::Batch>)->bool{
    let normal_values: Vec<_> = normal.values().collect();
    let patched_values: Vec<_> = patched.values().collect();

    
    if  normal.len() == patched.len(){
        'outer: for i in 0..normal_values.len(){
           for j in 0..patched_values.len(){
            if (normal_values[i].sequence == patched_values[j].sequence) && ((normal_values[i].state == themis_pbft::requests::BatchState::Open && patched_values[j].state == themis_patch_pbft::requests::BatchState::Open) || (normal_values[i].state == themis_pbft::requests::BatchState::Missing && patched_values[j].state == themis_patch_pbft::requests::BatchState::Missing) ||(normal_values[i].state == themis_pbft::requests::BatchState::Prepared && patched_values[j].state == themis_patch_pbft::requests::BatchState::Prepared)) {
                continue 'outer;
                }
            }
            println!("nv: {:?}", normal_values);
            println!("pv: {:?}", patched_values);
            return false;

        }
        return true;
    }
    print!("outer failure");
false
}

fn compare_log(normal: &themis_pbft::message_log::OrderingLog, patched: &themis_patch_pbft::message_log::OrderingLog)->bool{
    if normal.old_views.len() == patched.old_views.len(){
        if normal.current_view.len() == patched.current_view.len() {
            for i in 0..normal.current_view.slots.len()
            {
                let x = &normal.current_view.slots[i];
                match x {
                    Some(slot) => {
                        let y = &patched.current_view.slots[i];
                        match y {
                            Some(sl) => {
                                if (sl.state == themis_patch_pbft::message_log::OrderingState::Open && slot.state == themis_pbft::message_log::OrderingState::Open)||(sl.state == themis_patch_pbft::message_log::OrderingState::Prepared && slot.state == themis_pbft::message_log::OrderingState::Prepared)||(sl.state == themis_patch_pbft::message_log::OrderingState::Committed && slot.state == themis_pbft::message_log::OrderingState::Committed){
                                if  ((sl.commits.len() == slot.commits.len()) && (sl.prepares.len() == slot.prepares.len()) && (sl.pre_prepare.is_some() == slot.pre_prepare.is_some())){
                               
                                    continue;
                                }
                                else {
                                    return false
                                }
                                
                            }
                                else {
                                
                                    return false
                                }

                        }
                            None=>{return true}
                        }
                    }

                    None =>{return true}
                }

            }
            return true;
                
        }
    }
    ;
    false
}

fn compare_checkpoint(normal: &themis_pbft::checkpointing::Checkpointing, patched: &themis_patch_pbft::checkpointing::Checkpointing)->bool{
    
    if normal.checkpoints.len() == patched.checkpoints.len(){
        if normal.checkpoints.keys().eq(patched.checkpoints.keys()){
            return true
        }
        
    }
    return false;
}

pub fn compare_versions(pbft: &PBFTContext, patch_pbft: &PBFTPatchContext, data: &[u8], message_type: u64, source: u64, destination: u64, view: u64) -> bool{

    let normal = &pbft.pbft;
    let patched = &patch_pbft.pbft;
    
    if normal.id() != patched.id() {
        log_on_comp(message_type, source, destination, view, "id");        
        exit(0);
    }
    if normal.low_mark() != patched.low_mark() {
        log_on_comp(message_type, source, destination, view, "low_mark");        
        exit(0);
    }
    if normal.next_sequence() != patched.next_sequence(){
        log_on_comp(message_type, source, destination, view, "next_sequence");        
        exit(0);
    }
    if normal.last_commit != patched.last_commit {
        log_on_comp(message_type, source, destination, view, "last_commit");        
        exit(0);
    }
    if normal.view() != patched.view() {
        log_on_comp(message_type, source, destination, view, "view");        
        exit(0);
    }
    if !compare_state(&normal.state, &patched.state) {
        
        log_on_comp(message_type, source, destination, view, "state");        
        exit(0);

    }
    if !compare_request(&normal.requests.requests, &patched.requests.requests){
        log_on_comp(message_type, source, destination, view, "requests");       
         exit(0);
    }
    if !compare_request_batch(&normal.requests.instances, &patched.requests.instances){
        log_on_comp(message_type, source, destination, view, "instances");        
        exit(0);
    }
    if !compare_log(&normal.log, &patched.log){
        log_on_comp(message_type, source, destination, view, "log");        
        exit(0);
    }
    if !compare_checkpoint(&normal.checkpointing, &patched.checkpointing){
        log_on_comp(message_type, source, destination, view, "checkpointing");
        exit(0);
    }

    false
}
