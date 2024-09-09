use std::{fs::OpenOptions, io, path::PathBuf};

use dirs::{config_dir, home_dir};

struct ConfigOptions{
    filesystem_location:PathBuf,
}

fn main() -> std::io::Result<()> {
    let config_dir = config_dir().unwrap(); 
    let config_path: PathBuf = [config_dir.to_str().unwrap(),"walrust.conf"].iter().collect();
    let mut config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(config_path);
   

    Ok(())
     
}