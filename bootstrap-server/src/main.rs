use std::fs::{create_dir_all, File, OpenOptions};

use daemonize::Daemonize;
use protocol;




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
