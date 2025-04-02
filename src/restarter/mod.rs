use std::{fs, path::{Path, PathBuf}, process::{exit, Command}};

const LAST_PATCH_PATH_FILE: &str = "/home/stefan/Projects/themisfuzzer/themisfuzzer/tmp/last_patch_path.txt";

fn save_last_applied_patch(path: &PathBuf) {
    if let Err(e) = fs::write(LAST_PATCH_PATH_FILE, path.to_str().unwrap()) {
        eprintln!("Failed to save last applied patch: {}", e);
    }
}

pub fn load_last_applied_patch() -> Option<PathBuf> {
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
                return Some(entry.path()); 
            }
        }
    }
    None
}

fn restart_fuzzing() {
    let _status = Command::new("sh")
    .arg("-c")
    .arg("cargo fuzz run fuzz_target_1")
    .status()
    .expect("Failed to restart fuzzer");
}


pub fn apply_patches(){

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
                    let _rev = Command::new("git").args(["-C", themis_path,"apply","--reverse", "--whitespace=fix", &p.display().to_string()]).status().expect("Failed to apply patch");
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
    
            restart_fuzzing()
            
        }
        None => {
            println!("dead");
            exit(0)}
    }
    
}
