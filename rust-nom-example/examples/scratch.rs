use nom;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::multi::*;
use nom::sequence::*;
use nom::Err;
use nom::IResult;

type Res<'a> = IResult<&'a str, &'a str>;

fn main() {}
