use std::collections::LinkedList;
use std::io::{Read, Write};
use std::str::FromStr;

pub enum EofBehavior {
    Zero,
    Neg1,
    NoChange,
}

impl FromStr for EofBehavior {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zero" => Ok(Self::Zero),
            "neg1" => Ok(Self::Neg1),
            "nc" => Ok(Self::NoChange),
            _ => Err(()),
        }
    }
}

static SRC_CHARSET: &[u8; 8] = b"<>+-,.[]";

pub fn minimize(src: &str) -> String {
    let mut output = String::new();
    for b in src.as_bytes() {
        if SRC_CHARSET.contains(b) {
            output.push(char::from(*b));
        }
    }
    output
}

pub trait ReadOneByte<R>
where
    R: Read,
{
    fn read_1_byte(&mut self) -> std::io::Result<u8>;
}

pub trait WriteOneByte<W>
where
    W: Write,
{
    fn write_1_byte(&mut self, byte: u8) -> std::io::Result<()>;
}

impl<R> ReadOneByte<R> for R
where
    R: Read,
{
    fn read_1_byte(&mut self) -> std::io::Result<u8> {
        let mut b = [0_u8; 1];
        self.read_exact(&mut b)?;
        Ok(b[0])
    }
}

impl<W> WriteOneByte<W> for W
where
    W: Write,
{
    fn write_1_byte(&mut self, byte: u8) -> std::io::Result<()> {
        self.write_all(&[byte])
    }
}

pub fn check_brackets(s: &str) -> bool {
    let mut stack = LinkedList::new();
    for b in s.as_bytes() {
        match *b {
            b'[' => stack.push_back(b'['),
            b']' => match stack.back() {
                None => return false,
                Some(b) if *b == b'[' => {
                    stack.pop_back().unwrap();
                }
                _ => return false,
            },
            _ => {}
        }
    }
    stack.is_empty()
}
