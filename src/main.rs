#[macro_use]
extern crate nom;

use nom::IResult;

#[derive(Debug)]
struct KvPair {
    key   : Vec<u8>,
    value : Vec<u8>,
}

named!(parse_pair<&[u8], KvPair>,
    chain!(
        k: take_until!(b"=")  ~ tag!(b"=") ~
        v: take_until!(b"\0") ~ tag!(b"\0"),
        || {
            KvPair {key: k.to_vec(), value: v.to_vec()}
        })
);

fn main()
{
    let p = parse_pair(b"asdf=fdsa\0");

    match p {
        IResult::Done(_, output)    => println!("Header: {:?} ", output),
        IResult::Error(err)         => println!("Error: {:?}", err),
        IResult::Incomplete(needed) => println!("Needed {:?}", needed),
    }
}
