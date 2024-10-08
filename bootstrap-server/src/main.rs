use std::{fs::{self, create_dir_all, File, OpenOptions}, net::SocketAddr, path::PathBuf};

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

    if let Some(parent_dir) = dir.parent() {
        // Create directories if they don't exist
        fs::create_dir_all(parent_dir)?;
    }

    let routing_table = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(dir);


    axum::serve(listener,app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    Ok(())

}

#[derive(Clone)]
struct AppState {
    local_addr: SocketAddr,
}

async fn handler(ConnectInfo(remote_addr): ConnectInfo<SocketAddr>) -> String {
    format!("Client IP: {}",remote_addr.ip())
}

async fn addToNetwork(){
     
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
