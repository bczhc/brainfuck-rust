use brainfuck::errors::*;
use brainfuck::{Cell, ReadOneByte};
use brainfuck::{CellSize, EofBehavior, Specifications};
use std::io::{ErrorKind, Read, Write};

pub fn start<R, W>(src: &str, reader: &mut R, writer: &mut W, specs: &Specifications) -> Result<()>
where
    R: Read,
    W: Write,
{
    let mut runner = Runner::new(src, reader, writer, specs);
    match specs.cell_bits {
        CellSize::U8 => runner.run::<u8>(),
        CellSize::U16 => runner.run::<u16>(),
        CellSize::U32 => runner.run::<u32>(),
        CellSize::U64 => runner.run::<u64>(),
    }
}

struct Runner<'a, R, W>
where
    R: Read,
    W: Write,
{
    src: &'a str,
    reader: &'a mut R,
    writer: &'a mut W,
    specs: &'a Specifications,
}

impl<'a, R, W> Runner<'a, R, W>
where
    R: Read,
    W: Write,
{
    fn new(src: &'a str, reader: &'a mut R, writer: &'a mut W, specs: &'a Specifications) -> Self {
        Self {
            src,
            reader,
            writer,
            specs,
        }
    }

    fn run<N>(&mut self) -> Result<()>
    where
        N: CellType,
        <N as std::convert::TryFrom<u8>>::Error: std::fmt::Debug,
    {
        let mut data_cursor = DataCursor::<N>::new();
        let mut inst_cursor = StringCursor::new(self.src);

        loop {
            match inst_cursor.current() {
                b'<' => data_cursor.move_left(),
                b'>' => data_cursor.move_right(),
                b'+' => data_cursor.increase(),
                b'-' => data_cursor.decrease(),
                b',' => data_cursor.read_and_set(self.reader, &self.specs.eof_behavior),
                b'.' => data_cursor.print(self.writer)?,
                b'[' => {
                    if data_cursor.current() == N::default() {
                        inst_cursor.set_position(find_paired_right_bracket(
                            self.src,
                            inst_cursor.position(),
                        ));
                    }
                }
                b']' => {
                    if data_cursor.current() != N::default() {
                        inst_cursor.set_position(find_paired_left_bracket(
                            self.src,
                            inst_cursor.position(),
                        ))
                    }
                }
                _ => {}
            }
            inst_cursor.move_next();
            if inst_cursor.position() >= inst_cursor.len() {
                break;
            }
        }

        Ok(())
    }
}

fn find_paired_left_bracket(src: &str, position: usize) -> usize {
    let bytes = src.as_bytes();
    let mut count = 0_usize;
    for i in (0..position).rev() {
        match bytes[i] {
            b']' => count += 1,
            b'[' => {
                if count == 0 {
                    return i;
                }
                count -= 1;
            }
            _ => {}
        }
    }
    unreachable!()
}

fn find_paired_right_bracket(src: &str, position: usize) -> usize {
    let bytes = src.as_bytes();
    let mut count = 0_usize;

    for (i, b) in bytes.iter().enumerate().skip(position + 1) {
        match b {
            b'[' => count += 1,
            b']' => {
                if count == 0 {
                    return i;
                }
                count -= 1;
            }
            _ => {}
        }
    }
    unreachable!()
}

trait CellType: Cell + Default + Clone + Copy + TryFrom<u8> + Eq + PartialEq {}

macro_rules! impl_cell_type {
    ($x:ty) => {
        impl CellType for $x {}
    };
}

impl_cell_type!(u8);
impl_cell_type!(u16);
impl_cell_type!(u32);
impl_cell_type!(u64);

struct DataCursor<N>
where
    N: CellType,
{
    vec: Vec<N>,
    pos: usize,
}

impl<N> DataCursor<N>
where
    N: CellType,
{
    fn new() -> Self {
        Self {
            vec: vec![N::default(); 100],
            pos: 0,
        }
    }

    fn move_right(&mut self) {
        self.pos += 1;
        let capacity = self.vec.capacity();
        if self.pos >= capacity {
            self.vec.resize(capacity * 2, N::default());
        }
    }

    fn move_left(&mut self) {
        self.pos -= 1;
        if self.pos * 2 < self.vec.capacity() {
            self.vec.shrink_to(self.pos + 1)
        }
    }

    fn increase(&mut self) {
        let x = self.vec.get_mut(self.pos).unwrap();
        x.overflowing_inc_assign();
    }

    fn decrease(&mut self) {
        let x = self.vec.get_mut(self.pos).unwrap();
        x.overflowing_dec_assign();
    }

    fn print<W>(&self, out: &mut W) -> Result<()>
    where
        W: Write,
    {
        self.vec[self.pos].print_char(out)?;
        out.flush()?;

        Ok(())
    }

    fn read_and_set<R>(&mut self, reader: &mut R, eof_behavior: &EofBehavior)
    where
        R: Read,
        <N as std::convert::TryFrom<u8>>::Error: std::fmt::Debug,
    {
        let result = reader.read_1_byte();
        let read = match result {
            Ok(read) => N::try_from(read).unwrap(),
            Err(ref e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    match eof_behavior {
                        EofBehavior::Zero => N::default(),
                        EofBehavior::Neg1 => {
                            let mut n = N::default();
                            n.overflowing_dec_assign();
                            n
                        }
                        EofBehavior::NoChange => self.vec[self.pos],
                    }
                } else {
                    result.unwrap();
                    unreachable!();
                }
            }
        };
        self.vec[self.pos] = read;
    }

    fn current(&self) -> N {
        self.vec[self.pos]
    }
}

struct StringCursor<'a> {
    s: &'a str,
    pos: usize,
}

impl<'a> StringCursor<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, pos: 0 }
    }

    fn current(&self) -> u8 {
        self.s.as_bytes()[self.pos]
    }

    fn move_next(&mut self) {
        self.pos += 1;
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn set_position(&mut self, position: usize) {
        self.pos = position;
    }

    fn len(&self) -> usize {
        self.s.len()
    }
}
