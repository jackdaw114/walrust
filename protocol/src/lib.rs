#[allow(dead_code)]

pub mod protocol;

pub use protocol::*;



#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn malformed_header(){
        let malformed_header = "this\r\n is a \r\n\r\n test";    
        let parse_header = Protocol::header_parser(malformed_header) ;
        assert!(parse_header.is_err());
        match parse_header{
            Err(x) => assert_eq!(x,ProtocolError::MalformedHeader),
            Ok(_) => panic!("expected error"),
        }
    }
    #[test]
    fn malformed_payload(){
        let malformed_payload = "this\r\n is a \r\n\n test";    
        let parse_payload = Protocol::header_parser(malformed_payload) ;
        assert!(parse_payload.is_err());
        match parse_payload{
            Err(x) => assert_eq!(x,ProtocolError::MalformedPayload),
            Ok(_) => panic!("expected error"),
        }
    }
    #[test]
    fn valid_header_options_values(){
        let payload = "request_type:join\r\n\r\n";    
        let res = Protocol::header_parser(payload) ;
        assert!(res.is_ok());
    }
}
