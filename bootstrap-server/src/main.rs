use std::{collections::HashMap, default, error::Error, fs::{ create_dir_all,  OpenOptions}, io::{self,BufRead}, net::SocketAddr, path::{Path, PathBuf}, sync::{Arc, Mutex}};


use axum::{
    extract::{ConnectInfo, Query}, routing::get, Router
};
use dirs::data_dir;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use serde::Deserialize;

#[derive(Debug)]
struct RouteEntry {
    ip: String,
    port: u32,
    role: String, // change this to an enum 
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0,0,0,0],8000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    

    
    let dir:PathBuf = [data_dir().unwrap().to_str().unwrap(),"walrust","routing_table.txt"].iter().collect();
    println!("{:?}",dir);

    if let Some(parent_dir) = dir.parent() {
        // Create directories if they don't exist
        create_dir_all(parent_dir)?;
    }
    
    let routing_table = Arc::new(Mutex::new(loadRoutingTable(&dir)?)) ;
    
/*    println!("Parsed data:");
    for (hash, entry) in routing_table.iter() {
        println!("Hash: {}", hash);
        println!("  IP: {}", entry.ip);
        println!("  Port: {}", entry.port);
        println!(); // Add a blank line between entries
    }
    println!("Total entries: {}", routing_table.len());
    */
    let app = Router::new()
        .nest_service("/", ServeDir::new("dist"))
        .route("/test", get(handler))
        .route("/join-network", get(add_to_network))
        .with_state(routing_table);
    axum::serve(listener,app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    Ok(())

}


async fn handler(ConnectInfo(remote_addr): ConnectInfo<SocketAddr>) -> String {
    format!("Client IP: {}",remote_addr.ip())
}

#[derive(Deserialize)]
struct JoinParams{
    port: Option<String>,
    role: Option<String>
}

async fn add_to_network(
    Query(params): Query<JoinParams>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    routing_table:Arc<Mutex<HashMap<String,RouteEntry>>>
) -> String {
    let port= params.port.unwrap_or_else(|| "Anonymous".to_string());
    
    let route_map = routing_table.lock().unwrap();
    for (hash,entry) in route_map.iter(){
        // TODO: Check hash params here and then enter it into file
    }
    format!("Hello, {}! Your IP address is: {}", port, addr.ip())
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
