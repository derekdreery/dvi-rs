use std::io::{Write, self};
use nom::IResult;

/// A type that can be written to a stream (serialized)
pub trait Dump
{
    fn dump<W>(&self, &mut W) -> io::Result<()> where W: Write;
}

/// A type that can be parsed from a byte slice
pub trait Parse<V>
    where V: AsRef<[u8]>,
        Self: Sized
{
    /// Returns the Instruction and how many bytes were used
    fn parse(v: V) -> IResult<V, Self>;
}

