use std::{fs::OpenOptions, io, path::PathBuf};

use dirs::{config_dir, home_dir};

struct ConfigOptions{
    filesystem_location:PathBuf,
    filesystem_size:u64
}

fn main() -> std::io::Result<()> {
    let config_dir = config_dir().unwrap(); 
    let config_path: PathBuf = [config_dir.to_str().unwrap(),"walrust.conf"].iter().collect();
    println!("{:?}",config_path);
    let mut config_file = OpenOptions::new()
        .read(true)
        .create(true)
        .open(config_path);
   

    Ok(())
     
}

// To be included
// use std::{collections::HashMap, fs::File, io::Read};

// fn main() {
//     let mut file = File::open("conf_Test.txt").unwrap();
//     let mut data = String::new();
//     file.read_to_string(&mut data).unwrap();

//     // #[derive(Debug)]
//     struct NodeConfiguration {
//         f_size: u128,
//         author: String,
//         f_data: String
//     }

//     impl NodeConfiguration {
//         fn obtain_size(&mut self, string:&str) {
//             self.f_size = string[..string.len()-2].parse().unwrap(); //consider using .expect("unexpected size defined")
//         }

//         fn display_node(&self){
//             println!("author:{}\ndefined size:{}\ndata:{}", self.author, self.f_size, self.f_data)
//         }
//     }


//     let mut conf_dict: HashMap<&str, &str> = HashMap::new();
//     let mut skip_flag: bool = false;
//     for string in data.split_whitespace() {
//         //println!("{:?}", &string);
//         if string == "*/"{
//             skip_flag = false;
//             continue;
//         }
        
//         if string == "/*"{
//             skip_flag = true;
//         }
//         if skip_flag {
//             continue;
//         }
//         conf_dict.insert(&string[0..6], &string[7..]);
//     }

//     //println!("{:?}", conf_dict);

//     let mut new_node = NodeConfiguration{
//         f_size : 0,
//         author : conf_dict["author"].to_string(),
//         f_data : conf_dict["f_data"].to_string()
//     };

//     new_node.obtain_size(conf_dict["f_size"]);
//     new_node.display_node();
//     //println!("{:?}", new_node);

// }