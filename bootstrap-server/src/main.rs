use std::{collections::{BTreeMap}, fs::{ create_dir_all,  OpenOptions}, io::{self,BufRead}, net::SocketAddr, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use hyper::{ Error, StatusCode};
use serde_json::Value;
use sha1::{Sha1,Digest};
use std::any::type_name;
use axum::{body::Bytes, extract::{ConnectInfo, Multipart, Query, State}, http::response, response::IntoResponse, routing::{get, post}, Json, Router
};
use dirs::data_dir;
use tower_http::services::ServeDir;
use serde::{Deserialize, Serialize};


#[derive(Debug,Clone,Serialize)]
struct RouteEntry {
    ip: String,
    port: u32,
    node_name: String, // change this to an enum 
}

#[derive(Deserialize,Debug)]
struct FileParams{
    hash:Option<String>,
    file_name: Option<String>,
    node_name: Option<String>
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
    
    let app = Router::new()
        .nest_service("/", ServeDir::new("dist"))
        .route("/test", get(handler))
        .route("/join-network", get(add_to_network))
        .route("/add-file", post(upload_file))
        .route("/get-file", post(get_file))
        .route("/get-dir-structure", get(get_dir_structure))
        .with_state(routing_table);
    axum::serve(listener,app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    Ok(())

}


async fn handler(ConnectInfo(remote_addr): ConnectInfo<SocketAddr>) -> String {
    format!("Clent IP: {}",remote_addr.ip())
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

    let mut route_map = routing_table.lock().unwrap();
    let client_hash = hex::encode(result);
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

#[derive(Serialize)]
struct FileDetails{
    scope:String,
    file_name:String,
    data: String
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
        println!("before hashing file entry :{}{}",addr.ip().to_string(),file_name);
        hasher.update(format!("{}{}",addr.ip().to_string(),file_name).as_bytes());
        
        let result = hasher.finalize();

        let file_hash = hex::encode(result);
        println!("hashed file in db hash {}",file_hash);
        let mut route_map = routing_table.lock().unwrap();
        let mut change = None; 
        for (hash,entry) in route_map.iter(){
            println!("{},Entry:{:?}",hash,entry);
            println!("File hash: {}",file_hash);
            if file_hash == hash.to_string(){
                //update file
                return (StatusCode::OK,format!("File '{}' uploaded successfully", file_name));
            }
            if entry.port == 0{
                continue;
            }
            if file_hash > hash.to_string(){
                // store file in corresponding route
                
                println!("inside option {:?}",entry);
                change = Some((file_hash.clone(), RouteEntry{
                    ip: format!("{}:{}",entry.ip,entry.port),
                    port: 0,
                    node_name: addr.ip().to_string()
                }));
                let url = format!("{}:{}/add-file",entry.ip, entry.port);

                println!("{}",url);

                let res = ureq::post(&url)
                    .send_json(FileDetails{
                        scope: addr.ip().to_string(),
                        file_name:file_name.to_string(),
                        data:hex::encode(&data)
                    });
                println!("Response sent by node: {:?}",res);

                break;
            }

        }
        match change{
            Some((key,value)) =>{
                route_map.insert(key, value);
            },
            None => {
                let temp_entry = route_map.iter().find(|&(_,entry)| entry.port >0).unwrap().1.clone();
                println!("inside none of change {:?}",temp_entry);
                route_map.insert(file_hash, RouteEntry{
                    ip: format!("{}:{}",temp_entry.ip,temp_entry.port),
                    port: 0,
                    node_name: addr.ip().to_string()
                });
                let url = format!("http://{}:{}/add-file",temp_entry.ip, temp_entry.port);

                println!("{}",url);
                
                
                let res = ureq::post(&url)
                    .send_json(FileDetails{
                        scope: addr.ip().to_string(),
                        file_name:file_name.to_string(),
                        data:String::from_utf8(data.to_vec()).unwrap()
                    }).unwrap();
                println!("Response sent by node: {:?}",res);
            }
        }

        (StatusCode::OK,format!("File '{}' uploaded successfully", file_name))
    } else {
        (StatusCode::BAD_REQUEST,"No file uploaded".into())
    }
}

async fn get_file(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(routing_table):State<Arc<Mutex<BTreeMap<String,RouteEntry>>>>,
    Json(params):Json<FileParams>,
    ) ->impl IntoResponse{
    println!("{:?}",params);
    let mut route_map = routing_table.lock().unwrap(); 
    let file_name = params.file_name.unwrap_or_else(|| "Default.txt".to_string());
    let scope = params.node_name.unwrap_or_else(|| "Default.txt".to_string());
    let mut hasher = Sha1::new();

    println!("before hashing file entry :{}{}",scope,file_name);
    hasher.update(format!("{}{}",scope,file_name).as_bytes());
    let result = hasher.finalize();
    let file_hash = hex::encode(result);
    println!("comp hash {}",file_hash);
    for (hash,entry) in route_map.iter(){
        println!("get file {}",hash);
        if hash.to_string() == file_hash{
            println!("http://{}",&entry.ip);

            let res = ureq::post(&format!("http://{}/get-file",&entry.ip))
                .send_json(FileDetails{
                    scope:entry.node_name.clone(),
                    file_name:file_name.clone(),
                    data:"None".to_string()
                }).unwrap();
                
                
                println!("Response sent by node: {:?}",res);
                if res.has("Content-Length"){
                    let len:usize= res.header("Content-Length")
                        .unwrap()
                        .parse().unwrap();
                    let mut bytes: Vec<u8> = Vec::with_capacity(len);
                    res.into_reader()
                        .read_to_end(&mut bytes).unwrap();

                    let ret_data:Value = serde_json::from_str(&String::from_utf8(bytes).unwrap()).unwrap();
                    return Json(ret_data);
                }
                break;
        }
    }
    Json(Value::default())
}

async fn get_dir_structure(
    State(routing_table):State<Arc<Mutex<BTreeMap<String,RouteEntry>>>>
    ) -> impl IntoResponse{
    let lock = routing_table.lock().unwrap();
    let filtered_map: BTreeMap<String, RouteEntry> = lock.iter()
            .filter(|&(_, value)| value.port == 0)
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
    (StatusCode::OK,serde_json::to_string(&filtered_map).unwrap())
}

