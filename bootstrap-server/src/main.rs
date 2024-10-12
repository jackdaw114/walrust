use std::{collections::{BTreeMap}, fs::{ create_dir_all,  OpenOptions}, io::{self,BufRead}, net::SocketAddr, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use hyper::{ Error, StatusCode};
use sha1::{Sha1,Digest};
use std::any::type_name;
use axum::{extract::{ConnectInfo, Multipart, Query, State}, response::IntoResponse, routing::{get, post}, Router
};
use dirs::data_dir;
use tower_http::services::ServeDir;
use serde::Deserialize;


#[derive(Debug)]
struct RouteEntry {
    ip: String,
    port: u32,
    node_name: String, // change this to an enum 
}

#[derive(Deserialize)]
struct FileParams{
    hash:Option<String>,
    file_name: Option<String>
}

#[derive(Deserialize)]
struct JoinParams{
    port: Option<String>,
    role: Option<String>
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
        .route("/add-file", post(upload_file))
        .with_state(routing_table);
    axum::serve(listener,app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    Ok(())

}


async fn handler(ConnectInfo(remote_addr): ConnectInfo<SocketAddr>) -> String {
    format!("Clent IP: {}",remote_addr.ip())
}


fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

async fn add_to_network(
    Query(params): Query<JoinParams>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(routing_table):State<Arc<Mutex<BTreeMap<String,RouteEntry>>>>,
) -> String {
    let port= params.port.unwrap_or_else(|| "8000".to_string());
    let mut hasher = Sha1::new();
    print!("{}{}",addr.ip(),port);
    hasher.update(format!("{}{}",addr.ip(),port).as_bytes());
    let result = hasher.finalize();

    let client_hash = hex::encode(result);
    let mut route_map = routing_table.lock().unwrap();
    for (hash,entry) in route_map.iter(){
        println!("{}",hash);
        if client_hash == hash.to_string(){
            return "Your already on the network".to_string();
        }
    }
    route_map.insert(client_hash, RouteEntry{
        ip:addr.ip().to_string(),
        port:port.parse().unwrap_or_else(|_| 8000),
        node_name:"test".to_string()
    });
    //TODO: Add entry to routing table file here 
    println!("{:?}",route_map);
    format!("Hello, {}! Your IP address is: {}", port, addr.ip())
}

fn add_file(
        Query(params): Query<FileParams>,
        State(routing_table):State<Arc<Mutex<BTreeMap<String,RouteEntry>>>>,
    )->String{
 
    format!("added file to device with hash")
}


fn loadRoutingTable(filename: &PathBuf) -> io::Result<BTreeMap<String, RouteEntry>> {
    let path = Path::new(filename);
    let file = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(&path)?;
    let reader = io::BufReader::new(file);


    let mut hashmap = BTreeMap::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split('|').collect();
        
        if parts.len() == 4{
            let hash = parts[0].to_string();
            let port:u32 = parts[2].parse().expect("routing table has an issue with port numbers");
            let entry = RouteEntry {
                ip:parts[1].to_string(),
                port: port,
                node_name: parts[3].to_string()
            };

            hashmap.insert(hash, entry);
        }
        else {
            panic!("incorrect routing table");
        }
    }

    Ok(hashmap)
}

async fn upload_file(
    State(routing_table):State<Arc<Mutex<BTreeMap<String,RouteEntry>>>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut multipart:Multipart
    ) -> impl IntoResponse{
    if let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Length of `{}` is {} bytes", name, data.len());
        println!("File name: {}", file_name);
        println!("Content type: {}", content_type);
        
         
        let mut hasher = Sha1::new();
         
        hasher.update(format!("{}{}",addr.ip(),file_name).as_bytes());
        
        let result = hasher.finalize();

        let file_hash = hex::encode(result);
        let mut route_map = routing_table.lock().unwrap();
        let mut change = None; 
        for (hash,entry) in route_map.iter(){
            println!("{}",hash);
            if file_hash == hash.to_string(){
                //update file
                break;
            }
            if entry.port == 0{
                continue;
            }
            if file_hash > hash.to_string(){
                // store file in corresponding route
                
                change = Some((file_hash, RouteEntry{
                    ip: format!("{}:{}",entry.ip,entry.port),
                    port: 0,
                    node_name: addr.ip().to_string()
                }));
                let url = format!("{}:{}/add-file",entry.ip, entry.port);
                println!("{}",url);

                let res = ureq::post(&url)
                    .send_bytes(&data);
                println!("Response sent by node: {:?}",res);

                break;
            }
        }
        match change{
            Some((key,value)) =>{
                route_map.insert(key, value);
            },
            None => {}
        }

        println!("{}",String::from_utf8_lossy(&data));
        (StatusCode::OK,format!("File '{}' uploaded successfully", file_name))
    } else {
        (StatusCode::BAD_REQUEST,"No file uploaded".into())
    }
}

async fn get_file(
    Query(params):Query<FileParams>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(routing_table):State<Arc<Mutex<BTreeMap<String,RouteEntry>>>>
    ) ->String{
    let mut route_map = routing_table.lock().unwrap(); 
    let file_name = params.file_name.unwrap_or_else(|| "Default.txt".to_string());
    let mut hasher = Sha1::new();
    hasher.update(format!("{}{}",addr.ip(),file_name));
    let result = hasher.finalize();
    let file_hash = hex::encode(result);
    for (hash,entry) in route_map.iter(){
        println!("{}",hash);
        if hash.to_string() == file_hash{
            let res = ureq::get(&entry.ip).call().unwrap();
            return res.into_string().unwrap();
        }
    }
    "File not found".to_string()
}

