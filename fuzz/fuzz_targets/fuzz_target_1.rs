#![no_main]

use libfuzzer_sys::fuzz_target;
use themisfuzzer::to_fuzz;
use themisfuzzer::context;
use themisfuzzer::context::setup_with_checkpoint;
use themisfuzzer::context::PBFTContext;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use themisfuzzer::patch::to_fuzz_patch;
use themis_patch_pbft::test::setup_with_checkpoint_patch;
use themis_patch_pbft::test::PBFTPatchContext;
use rand::Rng;
use tokio::runtime::Runtime;


static PBFT_INSTANCE: Lazy<Mutex<PBFTContext>> = Lazy::new(|| {
    let pbft_context = context::setup_with_checkpoint(true, 10000);
   
    Mutex::new(pbft_context)
});

static PBFT_PATCH_INSTANCE: Lazy<Mutex<PBFTPatchContext>> = Lazy::new(|| {
    let pbft_patch_context = themis_patch_pbft::test::setup_with_checkpoint_patch(true, 10000);
  
    Mutex::new(pbft_patch_context)
});

static SEQ:Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));






fn harness(data: &[u8], sequence: u64, pbft: &mut PBFTContext, pbft_patch: &mut PBFTPatchContext, source:u64, destination:u64){
    let mut seq = SEQ.lock().unwrap();
    
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        tokio::join!(
            to_fuzz(data, *seq, pbft, source, destination),
            to_fuzz_patch(data, *seq, pbft_patch, source, destination)
        );
    });
    *seq += 1;
}



fuzz_target!(|data: &[u8]| {

    let mut rng = rand::thread_rng();
    let mut pbft_context = PBFT_INSTANCE.lock().unwrap();
    let mut pbft_patch_context = PBFT_PATCH_INSTANCE.lock().unwrap();

    let mut source = rng.gen_range(0..=1);
    let mut destination = rng.gen_range(0..=1);
    if rng.gen_bool(0.001){
        source = rng.r#gen();
        destination = rng.r#gen();
    }

    if data.len() >= 8{
        let mut sequence = u64::from_le_bytes(data[..8].try_into().unwrap());
        let payload = &data[8..];

        harness(payload, sequence, &mut pbft_context, &mut pbft_patch_context, source, destination);
        }
    else{
        let sequence = rng.r#gen(); 
        let payload = data;
        harness(payload, sequence, &mut pbft_context, &mut pbft_patch_context, source, destination);
    }


   
});
