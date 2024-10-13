use std::{net::SocketAddr, path::{PathBuf}, sync::Arc};

use axum::{body::{Body, Bytes}, extract::{self, Query}, response::IntoResponse, routing::post, Extension, Json, Router};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::{fs::{self, create_dir_all, File}, io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};

use dirs::data_dir;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0,0,0,0],3000));
    let listener = TcpListener::bind(&addr).await.unwrap();
    
    let shared_dir:Arc<PathBuf> =Arc::new( [data_dir().unwrap().to_str().unwrap(),"walrust"].iter().collect());
    fs::create_dir_all(&*shared_dir).await.unwrap();
    let app = Router::new()
        .route("/add-file", post(create_file))
        .route("/get-file", post(get_file))
        //.route("/files/:filename/write", post(write_to_file))
        .layer(Extension(shared_dir));
   axum::serve(listener,app.into_make_service())
        .await
        .unwrap();
    Ok(())

}

#[derive(Deserialize,Debug,Serialize)]
struct FileDetails{
    scope:String,
    file_name:String,
    data: String
}


async fn create_file(
    Extension(shared_dir): Extension<Arc<PathBuf>>,
    extract::Json(body):extract::Json<FileDetails>,
) -> impl IntoResponse {
    println!("Extension is {:?}",shared_dir);
    println!("Body:={:?}",body);
    let file_path = shared_dir.join([body.scope,body.file_name].iter().collect::<PathBuf>());
    println!("Path is {:?}",file_path);
    if let Some(parent_dir) = file_path.parent() {
        // Create directories if they don't exist
        create_dir_all(parent_dir).await.unwrap();
        
    }
    let Ok(mut file) = fs::File::create(file_path).await else{
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error creating file"));
    };
    file.write_all(body.data.as_bytes()).await.unwrap();
    (StatusCode::OK,"file saved".to_string()) 
}

async fn get_file(
    Extension(shared_dir): Extension<Arc<PathBuf>>,
    extract::Json(body):extract::Json<FileDetails>,
    ) -> impl IntoResponse{
    let file_path = shared_dir.join([&body.scope,&body.file_name].iter().collect::<PathBuf>());
    
    let mut file = File::open(file_path).await.unwrap();
    let mut contents=String::new();
    file.read_to_string(&mut contents).await.unwrap();

    Json(FileDetails{
        scope:body.scope,
        file_name:body.file_name,
        data:contents
    })
}
