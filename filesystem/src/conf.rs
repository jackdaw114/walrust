//#![allow(dead_code, unused_variables)]

use std::{fs::File, io::{self, BufRead}, path::PathBuf};


#[derive(Debug)]
pub struct UserConfig {
    f_size : u128,
    f_location: PathBuf,
    conf_location: PathBuf,
    s_block_location: u128,
    startpoint: u128,
    endpoint: u128
}

impl Default for UserConfig {

    fn default() -> Self {
        UserConfig {
            f_size : 1000,
            f_location: PathBuf::from("path/to/file"),
            conf_location: PathBuf::from("conf_Test.txt"),
            s_block_location: 8000,
            startpoint: 0010,
            endpoint: 1010
        }
    }
}

impl UserConfig {

    fn check_path(&self, f_path:&str, line_number:usize)-> PathBuf {
        let file_path = PathBuf::from(f_path);
        if file_path.exists() {
            return file_path
        }
        println!("[Line {:?}] {:?} does not exist. Setting default path", line_number, file_path);
        return PathBuf::from("default/path/on/device")
    }
    
    pub fn set_struct(&mut self) {
        let mut skip_flag = false;

        if let Ok(file) = File::open(&self.conf_location) {
            let reader = io::BufReader::new(file);
            
            for (line_number, line) in reader.lines().enumerate() {
                match line {
                    Ok(content) => {
                        if !content.is_empty() {
                            // println!("{}", content);
                            if content.contains("*/") {
                                skip_flag = false;
                                continue;
                            }
                        
                            if content.contains("/*") {
                                skip_flag = true;
                            }
                            if skip_flag {
                                continue;
                            }
                            let (key, value) = content.split_once("=").unwrap();
                            let (key, value) = (key.trim(), value.trim());
                            // println!("key:{:?} || value:{:?}", key, value);

                            // Assigning values to the struct

                                match key {
                                    "f_size" => match value.parse::<u128>() {
                                        Ok(value) => {
                                            self.f_size = value;
                                        }
                                        Err(e) => println!("[Line {:?}] {:?} value type is incorrect, ERROR is {:?}", line_number, key, e)
                                    },
                                    "startpoint" => match value.parse::<u128>() {
                                        Ok(value) => {
                                            self.startpoint = value;
                                        }
                                        Err(e) => println!("[Line {:?}] {:?} value type is incorrect, ERROR is {:?}", line_number, key, e)
                                    },
                                    "endpoint" => match value.parse::<u128>() {
                                        Ok(value) => {
                                            self.endpoint = value;
                                        }
                                        Err(e) => println!("[Line {:?}] {:?} value type is incorrect, ERROR is {:?}", line_number, key, e)
                                    },
                                    "s_block_location" => match value.parse::<u128>() {
                                        Ok(value) => {
                                            self.s_block_location = value;
                                        }
                                        Err(e) => println!("[Line {:?}] {:?} value type is incorrect, ERROR is {:?}", line_number, key, e)
                                    },
                                    "f_location" => self.f_location = self.check_path(value, line_number),

                                    "conf_location" => self.conf_location = self.check_path(value, line_number),

                                    _ => println!("[Line {:?}] `{}:{}` is an invalid key-value pair! (Read config docs for more information)", line_number, key, value)
                                }
                            
                        }
                    }
                        
                    Err(err) => eprintln!("Error reading line: {}", err),
                }
            }
            println!("{:?}", self);
        }
        else {
            eprintln!("Failed to open file {:?}", &self.conf_location);
        }
    }
}