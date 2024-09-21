mod conf;
use std::{fs::OpenOptions, io, path::PathBuf};
use conf::UserConfig;
use dirs::{config_dir, home_dir};

struct ConfigOptions{
    filesystem_location:PathBuf,
    filesystem_size:u64
}

fn main() -> std::io::Result<()> {
    //Cal coded
    let mut new_user = UserConfig::default();
    new_user.set_struct();
    new_user.block_space(3); // creates a text file of 3MB inorder to 'reserve' that amount of space
    let config_dir = config_dir().unwrap(); 
    let config_path: PathBuf = [config_dir.to_str().unwrap(),"walrust.conf"].iter().collect();
    println!("{:?}",config_path);
    let mut config_file = OpenOptions::new()
        .read(true)
        .create(true)
        .open(config_path);
   

    Ok(())
     
}