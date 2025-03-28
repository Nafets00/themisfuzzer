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


static PBFT_INSTANCE: Lazy<Mutex<PBFTContext>> = Lazy::new(|| {
    let pbft_context = context::setup_with_checkpoint(true, 10000);
   
    Mutex::new(pbft_context)
});

static PBFT_BACKUP_INSTANCE: Lazy<Mutex<PBFTContext>> = Lazy::new(|| {
    let pbft_backup_context = context::setup_with_checkpoint(false, 10000);
   
    Mutex::new(pbft_backup_context)
});

static PBFT_PATCH_INSTANCE: Lazy<Mutex<PBFTPatchContext>> = Lazy::new(|| {
    let pbft_patch_context = themis_patch_pbft::test::setup_with_checkpoint_patch(true, 10000);
  
    Mutex::new(pbft_patch_context)
});

static PBFT_BACKUP_PATCH_INSTANCE: Lazy<Mutex<PBFTPatchContext>> = Lazy::new(|| {
    let pbft_backup_patch_context = themis_patch_pbft::test::setup_with_checkpoint_patch(false, 10000);
  
    Mutex::new(pbft_backup_patch_context)
});

static SEQ:Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));






fn harness(rnd_var: u64, data: &[u8], sequence: u64, pbft: &mut PBFTContext, pbft_patch: &mut PBFTPatchContext, pbft_backup: &mut PBFTContext, pbft_backup_patch: &mut PBFTPatchContext,source:u64, destination:u64){
    
    
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        tokio::join!(
            to_fuzz(rnd_var, data, sequence, pbft, pbft_backup, source, destination),
            to_fuzz_patch(rnd_var, data, sequence, pbft_patch, pbft_backup_patch, source, destination)
        );
    });
    
}



fuzz_target!(|data: &[u8]| {
    
    let mut pbft_context = PBFT_INSTANCE.lock().unwrap();
    let mut pbft_patch_context = PBFT_PATCH_INSTANCE.lock().unwrap();
    let mut pbft_backup_context = PBFT_BACKUP_INSTANCE.lock().unwrap();
    let mut pbft_backup_patch_context = PBFT_BACKUP_PATCH_INSTANCE.lock().unwrap();

    

    let mut seq = SEQ.lock().unwrap();
    let mut rng = rand::thread_rng();
    let rnd_var = rng.gen_range(0..=10);

    let mut source = rng.gen_range(0..=1);
    let mut destination = rng.gen_range(0..=1);
    if rng.gen_bool(0.001){
        source = rng.r#gen();
        destination = rng.r#gen();
    }
    if rng.gen_bool(0.001){
        let gen_sequence = rng.gen_range(0..=*seq);
    }
   
    harness(rnd_var, data, *seq, &mut pbft_context, &mut pbft_patch_context, &mut pbft_backup_context, &mut pbft_backup_patch_context,source, destination);
    compare_versions(&pbft_context, &pbft_patch_context);    
    *seq += 1;

   
});
