use rustc_serialize::{Decodable, Encodable};
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode, DecodingError};
use std::mem::{transmute, swap};

pub fn ser<T: Encodable>(data: T) -> Vec<u8> {
    let mut encoded: Vec<u8> = encode(&data, SizeLimit::Infinite).unwrap();
    let (u0, u1, u2, u3) = unsafe { transmute(encoded.len() as u32) };
    encoded.insert(0, u0);
    encoded.insert(1, u1);
    encoded.insert(2, u2);
    encoded.insert(3, u3);
    encoded
}

pub fn de<T: Decodable>(buf: &mut Vec<u8>) -> Result<Option<T>, DecodingError> {
    if buf.len() <= 5 { return Ok(None); }
    let len: u32 = unsafe { transmute( (buf[0], buf[1], buf[2], buf[3]) ) };
    if buf.len() < (len as usize + 4) { return Ok(None); }
    
    let com = try!( decode(&buf[4..len as usize + 4]) );
    
    let mut still = buf.split_off(len as usize + 4);
    swap(buf, &mut still);
    Ok(Some(com))
}

#[test]
fn test_ser_de() {
    use commands::ClientCommand;
    let ping = ClientCommand::Ping;
    let mut vec = ser(ping);
    println!("{:?}", &vec);
    let de: ClientCommand = de(&mut vec).unwrap().unwrap();
    assert_eq!(de, ping);
    assert!(vec.is_empty());
}
