use protocol;
use std::io::stdin;


fn main() {
    let  buffer = String::from("request_type:join\r\n hello\r\n\r\nnothing");
    match protocol::Protocol::header_parser(&buffer){
        Ok(x) => println!("{:?}",x),
        Err(x) => {
            println!("{:?}",x)
        }

    }
}
