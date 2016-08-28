#[macro_use]
extern crate nom;

#[derive(Debug)]
struct KvPair<'a> {
    key   : &'a [u8],
    value : &'a [u8],
}

#[derive(Debug)]
struct OfPartitionHdr<'a> {
    signature : u8,
    length    : u16,
    name      : &'a[u8]
}

#[derive(Debug)]
struct OfPartition<'a> {
    header  : OfPartitionHdr<'a>,
    pairs   : Vec<KvPair<'a>>,
}

use nom::{IResult, ErrorKind};
use std::num::Wrapping;

/*
 * Implements the OF partition header checksumming algorithm
 */
fn checksum(partition : &[u8]) -> u8
{
	let mut c_sum = Wrapping(partition[0]);
    let one = Wrapping(1); /* is this seriously the best way to do this? */

	/*
     * The 2nd byte of that partition is the checksum. When
     * calculating the sum we assume it's zero so it can be
     * skipped in the calculation.
     */
	for c in &partition[2..] {
        let i_sum = c_sum + Wrapping(*c);

        if i_sum < c_sum {
            c_sum = i_sum + one;
        } else {
            c_sum = i_sum;
        }
	}

    return c_sum.0;
}

named!(parse_pair<&[u8], KvPair>,
    chain!(
        k: take_until_and_consume!(b"=") ~
        v: take_until_and_consume!(b"\0"),
        || {
            KvPair {key: k, value: v}
        }
    )
);

named!(parse_part_data<&[u8], Vec<KvPair> >,
    fold_many0!(
        parse_pair,
        Vec::new(),
        |mut acc: Vec<_>, item| { acc.push(item); acc}
    )
);

/*
 * FIXME: this should be implementable using combinators, but the checksumming
 *        makes this a little annoying since it needs to be validated.
 *
 */
fn parse_header(input : &[u8]) -> IResult<&[u8], OfPartitionHdr>
{
    if input.len() < 16 {
        return IResult::Incomplete(nom::Needed::Size(16))
    }

    let calc = checksum(&input[..16]);

    let sig = input[0];
    let chk = input[1];
    let len = ((input[2] as u16) << 8) | (input[3] as u16);
    let name = &input[4..16];

    let rest = &input[16..];

    if calc == chk {
        let ret = OfPartitionHdr {
            signature: sig,
            length:    len,
            name:     name,
        };

        IResult::Done(rest, ret)
    } else {
        let err = nom::Err::Code(ErrorKind::Custom(0));

        IResult::Error(err)
    }
}

named!(parse_nvram<&[u8], Vec<OfPartition> >,
    many1!( /* there should be atleast one partition */
        chain!(
            hdr     : parse_header ~
            pairs   : parse_part_data,
            || {
                OfPartition {header : hdr, pairs : pairs}
            }
))));

fn main()
{
    let r = parse_nvram( b"\x51\xb5\x01\x00\x69\x62\x6d\x2c\x73\x6b\x69\x62\x6f\x6f\x74\x00asdf=fdsa\0test1=test2\0");
//    let q = parse_part_data(b"asdf=fdsa\0test1=test2\0");
//    let p = parse_header(b"\x51\xb5\x01\x00\x69\x62\x6d\x2c\x73\x6b\x69\x62\x6f\x6f\x74\x00");

    match r {
        IResult::Done(_, output)    => println!("Header: {:?} ", output),
        IResult::Error(err)         => println!("Parse Error: {:?}", err),
        IResult::Incomplete(needed) => println!("Needed {:?}", needed),
    }
}
