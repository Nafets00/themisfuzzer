use crate::restarter::load_last_applied_patch;


pub fn log_on_comp(message_type:u64, source:u64, destination:u64, view:u64, status:&str){
    let last_patch = load_last_applied_patch();
    println!("Found a differance for patch: {:?} for message type: {}, source: {}, destination {}, view: {}, at status: {}", last_patch, message_type, source, destination, view, status)
}
