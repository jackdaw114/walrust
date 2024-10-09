use std::{collections::HashMap, error::Error,  fs::{ create_dir_all,  OpenOptions}, io::{self,BufRead}, net::SocketAddr, path::{Path, PathBuf}};


use axum::{
    Router,
    routing::get,
    extract::ConnectInfo
};
use daemonize::Daemonize;
use dirs::data_dir;
use tower_http::services::ServeDir;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0,0,0,0],8000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    

    let app = Router::new()
        .nest_service("/", ServeDir::new("dist"))
        .route("/test", get(handler));
    
    let dir:PathBuf = [data_dir().unwrap().to_str().unwrap(),"walrust","routing_table.txt"].iter().collect();
    println!("{:?}",dir);
    
    let routing_table = loadRoutingTable(&dir).map_err(|err| {
        return err
    });
    
    println!("Parsed data:");
    for (hash, entry) in routing_table {
        println!("Hash: {}", hash);
        println!("  IP: {}", entry.ip);
        println!("  Port: {}", entry.port);
        println!("  Details: {}", entry.details);
        println!(); // Add a blank line between entries
    }
    println!("Total entries: {}", routing_table.len());
    

    if let Some(parent_dir) = dir.parent() {
        // Create directories if they don't exist
        create_dir_all(parent_dir)?;
    }



    axum::serve(listener,app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    Ok(())

}


async fn handler(ConnectInfo(remote_addr): ConnectInfo<SocketAddr>) -> String {
    format!("Client IP: {}",remote_addr.ip())
}

async fn addToNetwork(){
     
}


#[derive(Debug)]
struct RouteEntry {
    ip: String,
    port: u32,
    role: String, // change this to an enum 
}

fn loadRoutingTable(filename: &PathBuf) -> io::Result<HashMap<String, RouteEntry>> {
    let path = Path::new(filename);
    let file = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(&path)?;
    let reader = io::BufReader::new(file);


    let mut hashmap = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split('|').collect();
        
        if parts.len() == 4{
            let hash = parts[0].to_string();
            let port:u32 = parts[2].parse().expect("routing table has an issue with port numbers");
            let entry = RouteEntry {
                ip:parts[1].to_string(),
                port: port,
                role: parts[3].to_string()
            };

            hashmap.insert(hash, entry);
        }
        else {
            panic!("incorrect routing table");
        }
    }

    Ok(hashmap)
}
/*
fn main() -> std::io::Result<()> {
    let finch_dir = "/tmp/finch";
    if let Err(x) = create_dir_all(finch_dir){
        return Err(x);
    }
    let stdout = OpenOptions::new()
        .create(true)
        .write(true)
        .open(finch_dir.to_string() + "/finch-bootstrap_server.out")
        .unwrap();


    let stderr = OpenOptions::new()
        .create(true)
        .write(true)
        .open(finch_dir.to_string() + "/finch-bootstrap_server.err")
        .unwrap();
   

    let daemon = Daemonize::new()
        .pid_file("/tmp/test.pid") 
        .chown_pid_file(true)      
        .working_directory("/tmp") 
        .user("nobody")
        .group("daemon") //TODO: create a custom group with privilidges as well as a
                                       //custom user
        .group(2)        
        .umask(0o777)    
        .stdout(stdout)  
        .stderr(stderr)  
        .privileged_action(|| "Executed before drop privileges"); 


    match daemon.start(){
        Ok(_) => {
            println!("inside daemon");

        }
        Err(e) =>{
            eprintln!("Error, {}",e);
        }
    }

    Ok(())
}
*/
