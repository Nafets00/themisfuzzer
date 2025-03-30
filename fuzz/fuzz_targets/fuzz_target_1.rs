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
use std::process;
use std::process::{exit,Command, Stdio};
use std::time::{Duration, Instant};
use std::env;
use std::os::unix::process::CommandExt;
use std::fs;
use std::path::{Path,PathBuf};


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

//static LAST_APPLIED_PATCH_PATH: Lazy<Mutex<PathBuf>> = Lazy::new(|| Mutex::new(PathBuf::from("")));

static START_TIME: Lazy<Instant> = Lazy::new(|| Instant::now());  
static TIMEOUT: Lazy<Duration> = Lazy::new(|| Duration::from_secs(30));
static SEQ:Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));


const LAST_PATCH_PATH_FILE: &str = "/home/stefan/Projects/themisfuzzer/themisfuzzer/tmp/last_patch_path.txt";



fn harness(rnd_var: u64, data: &[u8], sequence: u64, pbft: &mut PBFTContext, pbft_patch: &mut PBFTPatchContext,pbft2: &mut PBFTContext, pbft_patch2: &mut PBFTPatchContext,source:u64, destination:u64){
    
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        tokio::join!(
            to_fuzz(rnd_var, data, sequence, pbft,pbft2, source, destination),
            to_fuzz_patch(rnd_var, data, sequence, pbft_patch, pbft_patch2,source, destination)
        );
    });
    
}

fn save_last_applied_patch(path: &PathBuf) {
    if let Err(e) = fs::write(LAST_PATCH_PATH_FILE, path.to_str().unwrap()) {
        eprintln!("Failed to save last applied patch: {}", e);
    }
}

fn load_last_applied_patch() -> Option<PathBuf> {
    if let Ok(saved_path) = fs::read_to_string(LAST_PATCH_PATH_FILE) {
        let trimmed = saved_path.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }
    None
}


fn is_directory_empty(path: &str) -> bool {
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(), 
        Err(_) => false, 
    }
}

fn get_first_file_path(dir_path: &str) -> Option<PathBuf> {
    let dir = Path::new(dir_path);
    if dir.is_dir() {
        if let Ok(mut entries) = fs::read_dir(dir) {
            if let Some(Ok(entry)) = entries.next() {
                return Some(entry.path()); // Get full path
            }
        }
    }
    None
}

fn restart_fuzzing() {
    println!("Recompiling the fuzz target...");

    let args: Vec<String> = env::args().collect();
    
    let status = Command::new("cargo")
        .args(["fuzz", "run", "fuzz_target_1"])
        .status()
        .expect("Failed to restart fuzzing process");

    if status.success() {
        println!("exit0 here");
        exit(0); // Exit old process after restart
    } else {
        eprintln!("Fuzzing restart failed.");
        exit(1);
    }
}


fn apply_patches(){

    let themis_path = "/home/stefan/ThemisPatch/";
    let patch_path = "/home/stefan/patches/successful_patches/";
    let applied_path ="/home/stefan/patches/applied_patches/";

    
    
    match get_first_file_path(patch_path){
        Some(path) => {
            if is_directory_empty(applied_path){
                println!("path: {}", &path.display());
                let status = Command::new("git").args(["-C", themis_path,"apply", "--whitespace=fix", &path.display().to_string()]).status().expect("Failed to apply patch");
                if status.success(){
                    println!("successfull");
                    let new_path = Path::new(applied_path).join(path.file_name().unwrap());
                    if let Err(e) = fs::rename(path, &new_path){
                        println!("Failed to move patch file: {}", e);
                    }
                    else{
                        println!("Patch file moved to {:?}", new_path);
                        save_last_applied_patch(&new_path);
                        
                    }
                }
                else{
                    println!("failure at status: {:?}", status);
                    
                }
            }
            else{
                if let Some(p) = load_last_applied_patch(){
                
                    println!("last app: {:?}", p.display());
                    let rev = Command::new("git").args(["-C", themis_path,"apply","--reverse", "--whitespace=fix", &p.display().to_string()]).status().expect("Failed to apply patch");
                }
                let status = Command::new("git").args(["-C", themis_path,"apply", "--whitespace=fix", &path.display().to_string()]).status().expect("Failed to apply patch");
                if status.success(){
                    let new_path = Path::new(applied_path).join(path.file_name().unwrap());
                    if let Err(e) = fs::rename(path, &new_path){
                        println!("Failed to move patch file: {}", e);
                    }
                    else{
                        println!("Patch file moved to {:?}", new_path);
                        save_last_applied_patch(&new_path);
                    }
                }
                
            }
    
            let args: Vec<String> = env::args().collect();
            restart_fuzzing()
            
        }
        None => {
            println!("ded");
            exit(0)}
    }
    
}


fuzz_target!(|data: &[u8]| {

    let elapsed = START_TIME.elapsed();
    
    if elapsed >= *TIMEOUT
    {
        apply_patches();
    }
    if data.len() > 5 {
        
        let mut pbft_context = PBFT_INSTANCE.lock().unwrap();
        let mut pbft_patch_context = PBFT_PATCH_INSTANCE.lock().unwrap();
        let mut pbft_backup_context = PBFT_BACKUP_INSTANCE.lock().unwrap();
        let mut pbft_backup_patch_context = PBFT_BACKUP_PATCH_INSTANCE.lock().unwrap();

        let msg_type:u64 = data[0].into();
        let source:u64 = data[1].into();
        let destination:u64 = data[2].into();
        let view:u64 = data[3].into();
        let mut use_replica = false;
        //println!("type: {}", msg_type);


        let mut seq = SEQ.lock().unwrap();
        let mut sequence = seq.clone();
        let mut rng = rand::thread_rng();
        let mut is_correct = false;

        if rng.gen_bool(0.1){
            sequence = rng.gen_range(0..=sequence);
        }
        if data[5] == 0x0{
            is_correct = true;
        }
        let data = &data[4..];

        if rng.gen_bool(0.5){
            use_replica = true;
        }


        if use_replica{
            harness(msg_type, data, sequence, &mut pbft_backup_context, &mut pbft_backup_patch_context,&mut pbft_context, &mut pbft_patch_context,source, destination);

        }
        else{
            harness(msg_type, data, sequence, &mut pbft_context, &mut pbft_patch_context,&mut pbft_backup_context, &mut pbft_backup_patch_context,source, destination);
        }

        //compare_versions(&pbft_context, &pbft_patch_context);    
        *seq += 1;
    }
});
        