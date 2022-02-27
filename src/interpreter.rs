use crate::errors::*;
use crate::{EofBehavior, Specifications};
use brainfuck::{ReadOneByte, WriteOneByte};
use std::io::{ErrorKind, Read, Write};

pub fn start<R, W>(src: &str, reader: &mut R, writer: &mut W, specs: &Specifications) -> Result<()>
where
    R: Read,
    W: Write,
{
    let mut data_cursor = DataCursor::new();
    let mut inst_cursor = StringCursor::new(src);

    loop {
        match inst_cursor.current() {
            b'<' => data_cursor.move_left(),
            b'>' => data_cursor.move_right(),
            b'+' => data_cursor.increase(),
            b'-' => data_cursor.decrease(),
            b',' => data_cursor.read_and_set(reader, &specs.eof_behavior),
            b'.' => data_cursor.print(writer)?,
            b'[' => {
                if data_cursor.current() == 0 {
                    inst_cursor
                        .set_position(find_paired_right_bracket(src, inst_cursor.position()));
                }
            }
            b']' => {
                if data_cursor.current() != 0 {
                    inst_cursor.set_position(find_paired_left_bracket(src, inst_cursor.position()))
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

struct DataCursor {
    vec: Vec<u8>,
    pos: usize,
}

impl DataCursor {
    fn new() -> Self {
        Self {
            vec: vec![0_u8; 100],
            pos: 0,
        }
    }

    fn move_right(&mut self) {
        self.pos += 1;
        let capacity = self.vec.capacity();
        if self.pos >= capacity {
            self.vec.resize(capacity * 2, 0);
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
        *x = x.overflowing_add(1).0;
    }

    fn decrease(&mut self) {
        let x = self.vec.get_mut(self.pos).unwrap();
        *x = x.overflowing_sub(1).0;
    }

    fn print<W>(&self, out: &mut W) -> Result<()>
    where
        W: Write,
    {
        out.write_1_byte(self.vec[self.pos])?;
        out.flush()?;
        Ok(())
    }

    fn read_and_set<R>(&mut self, reader: &mut R, eof_behavior: &EofBehavior)
    where
        R: Read,
    {
        let result = reader.read_1_byte();
        let read = match result {
            Ok(read) => read,
            Err(ref e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    match eof_behavior {
                        EofBehavior::Zero => 0,
                        EofBehavior::Neg1 => 0_u8.overflowing_sub(1).0,
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

    fn current(&self) -> u8 {
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
