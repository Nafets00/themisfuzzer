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
use themisfuzzer::comp::compare_versions;
use std::time::{Duration, Instant};
use themisfuzzer::restarter::apply_patches;
use std::process::exit;


static PBFT_INSTANCE: Lazy<Mutex<PBFTContext>> = Lazy::new(|| {
    let pbft_context = context::setup_with_checkpoint(true, 1000);
   
    Mutex::new(pbft_context)
});

static PBFT_BACKUP_INSTANCE: Lazy<Mutex<PBFTContext>> = Lazy::new(|| {
    let pbft_backup_context = context::setup_with_checkpoint(false, 1000);
   
    Mutex::new(pbft_backup_context)
});

static PBFT_PATCH_INSTANCE: Lazy<Mutex<PBFTPatchContext>> = Lazy::new(|| {
    let pbft_patch_context = themis_patch_pbft::test::setup_with_checkpoint_patch(true, 1000);
  
    Mutex::new(pbft_patch_context)
});

static PBFT_BACKUP_PATCH_INSTANCE: Lazy<Mutex<PBFTPatchContext>> = Lazy::new(|| {
    let pbft_backup_patch_context = themis_patch_pbft::test::setup_with_checkpoint_patch(false, 1000);
  
    Mutex::new(pbft_backup_patch_context)
});


static START_TIME: Lazy<Instant> = Lazy::new(|| Instant::now());  
static TIMEOUT: Lazy<Duration> = Lazy::new(|| Duration::from_secs(70*60));
static SEQ:Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(1));






async fn harness(
    rnd_var: u64,
    data: &[u8],
    view: u64,
    sequence: u64,
    pbft: &mut PBFTContext,
    pbft_patch: &mut PBFTPatchContext,
    pbft2: &mut PBFTContext,
    pbft_patch2: &mut PBFTPatchContext,
    source: u64,
    destination: u64,
) {

    let _ = to_fuzz_patch(rnd_var, data, view, sequence, pbft_patch, pbft_patch2, source, destination).await;
    let _ = to_fuzz(rnd_var, data, view, sequence, pbft, pbft2, source, destination).await;

}



fuzz_target!(|data: &[u8]| {

    let elapsed = START_TIME.elapsed();
    
    if elapsed >= *TIMEOUT
    {
        exit(0); //remove to start loop with patch applying, may crash at reload but new patch will still apply
        apply_patches();
    }
    if data.len() > 5{
        
        let mut pbft_context = PBFT_INSTANCE.lock().unwrap();
        let mut pbft_patch_context = PBFT_PATCH_INSTANCE.lock().unwrap();
        let mut pbft_backup_context = PBFT_BACKUP_INSTANCE.lock().unwrap();
        let mut pbft_backup_patch_context = PBFT_BACKUP_PATCH_INSTANCE.lock().unwrap();


        

        /* resets the log every 60 seconds
        if elapsed.as_secs()%60 == 0{
            pbft_context.pbft.log = themis_pbft::message_log::OrderingLog::new(pbft_context.pbft.config.high_mark_delta);
            pbft_patch_context.pbft.log = themis_patch_pbft::message_log::OrderingLog::new(pbft_patch_context.pbft.config.high_mark_delta);
            pbft_backup_context.pbft.log = themis_pbft::message_log::OrderingLog::new(pbft_backup_context.pbft.config.high_mark_delta);
            pbft_backup_patch_context.pbft.log = themis_patch_pbft::message_log::OrderingLog::new(pbft_backup_patch_context.pbft.config.high_mark_delta);
        }
        */

        let mut msg_type:u64 = data[0].into();
        msg_type = msg_type%13;
        let source:u64 = data[1].into();
        let destination:u64 = data[2].into();
        let view:u64 = data[3].into();
        let mut use_replica = false;


        let mut seq = SEQ.lock().unwrap();
        let mut sequence = seq.clone();
        let mut rng = rand::thread_rng();

        if rng.gen_bool(0.5){
            sequence = rng.gen_range(0..=sequence);
        }
        else{
             *seq += 1;
        }
        
        let data = &data[4..];

        if rng.gen_bool(0.5){
            use_replica = true;
        }
        

        tokio::runtime::Runtime::new().unwrap().block_on(async {
    if use_replica {
        harness(
            msg_type, data, view, sequence, 
            &mut pbft_backup_context, &mut pbft_backup_patch_context,
            &mut pbft_context, &mut pbft_patch_context, source, destination
        ).await;
    } else {
        harness(
            msg_type, data, view, sequence, 
            &mut pbft_context, &mut pbft_patch_context,
            &mut pbft_backup_context, &mut pbft_backup_patch_context, source, destination
        ).await;
    }
    });
    if use_replica{
        compare_versions(&pbft_backup_context, &pbft_backup_patch_context, data, msg_type, source, destination, view);
    }
    else{
        compare_versions(&pbft_context, &pbft_patch_context, data, msg_type, source, destination, view);
    }
        
  
}
});
        