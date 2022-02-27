use std::collections::LinkedList;
use std::io::{Read, Write};
use std::str::FromStr;

pub mod errors;

#[derive(Debug)]
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

pub trait WriteString<W>
where
    W: Write,
{
    fn write_str(&mut self, s: &str) -> std::io::Result<()>;

    fn new_line(&mut self) -> std::io::Result<()> {
        self.write_str("\n")
    }

    fn write_line(&mut self, line: &str) -> std::io::Result<()> {
        self.write_str(line)?;
        self.new_line()
    }
}

impl<W> WriteString<W> for W
where
    W: Write,
{
    fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        self.write_all(s.as_bytes())
    }
}

pub trait WriteChar {
    fn write_char(&mut self, c: char) -> std::io::Result<()>;
}

impl<W> WriteChar for W
where
    W: Write,
{
    fn write_char(&mut self, c: char) -> std::io::Result<()> {
        let mut buf = vec![0_u8; c.len_utf8()];
        c.encode_utf8(&mut buf);
        self.write_all(&buf)
    }
}

#[derive(Debug)]
pub enum CellSize {
    U8,
    U16,
    U32,
    U64,
}

impl CellSize {
    pub fn from_size(bits: u32) -> Option<Self> {
        match bits {
            u8::BITS => Some(Self::U8),
            u16::BITS => Some(Self::U16),
            u32::BITS => Some(Self::U32),
            u64::BITS => Some(Self::U64),
            _ => None,
        }
    }
}

impl FromStr for CellSize {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let bits = s.parse::<u32>();
        if bits.is_err() {
            return Err(());
        }
        let bits = bits.unwrap();
        match CellSize::from_size(bits) {
            None => Err(()),
            Some(s) => Ok(s),
        }
    }
}

#[derive(Debug)]
pub struct Specifications {
    pub cell_bits: CellSize,
    pub eof_behavior: EofBehavior,
}

pub trait Cell {
    fn overflowing_inc_assign(&mut self);
    fn overflowing_dec_assign(&mut self);
    fn print_char<W>(&self, output: &mut W) -> errors::Result<()>
    where
        W: Write;
}
impl Cell for u8 {
    fn overflowing_inc_assign(&mut self) {
        *self = self.overflowing_add(1).0;
    }

    fn overflowing_dec_assign(&mut self) {
        *self = self.overflowing_sub(1).0;
    }

    fn print_char<W>(&self, output: &mut W) -> errors::Result<()>
    where
        W: Write,
    {
        output.write_1_byte(*self)?;
        Ok(())
    }
}

impl Cell for u16 {
    fn overflowing_inc_assign(&mut self) {
        *self = self.overflowing_add(1).0;
    }

    fn overflowing_dec_assign(&mut self) {
        *self = self.overflowing_sub(1).0;
    }

    fn print_char<W>(&self, output: &mut W) -> errors::Result<()>
    where
        W: Write,
    {
        let cp = *self as u32;
        if cp < 0x10000 {
            cp.print_char(output)
        } else {
            unimplemented!()
        }
    }
}

impl Cell for u32 {
    fn overflowing_inc_assign(&mut self) {
        *self = self.overflowing_add(1).0;
    }

    fn overflowing_dec_assign(&mut self) {
        *self = self.overflowing_sub(1).0;
    }

    fn print_char<W>(&self, output: &mut W) -> errors::Result<()>
    where
        W: Write,
    {
        match char::from_u32(*self) {
            None => Err(errors::Error::InvalidUnicode),
            Some(c) => {
                output.write_char(c)?;
                Ok(())
            }
        }
    }
}
impl Cell for u64 {
    fn overflowing_inc_assign(&mut self) {
        *self = self.overflowing_add(1).0;
    }

    fn overflowing_dec_assign(&mut self) {
        *self = self.overflowing_sub(1).0;
    }

    fn print_char<W>(&self, output: &mut W) -> errors::Result<()>
    where
        W: Write,
    {
        let u = u32::try_from(*self)?;
        u.print_char(output)
    }
}

#[allow(unused)]
pub fn surrogate_pair_to_unicode(lead: u16, trail: u16) -> u32 {
    (((((lead - 0xd800_u16) & 0b11_1111_1111_u16) as u32) << 10u32)
        | (((trail - 0xdc00_u16) & 0b11_1111_1111_u16) as u32))
        + 0x10000
}
