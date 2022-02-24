#![feature(mixed_integer_ops)]

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::fs::{read, write, File};
use std::io::{stdin, stdout, Read, Stdout, Write};
use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;

// chars: <>+_.,[]
fn main() {
    let src_path = "/home/bczhc/code/brainfuck/hello.bf";
    let mut src_file = File::open(src_path).unwrap();

    let mut src = String::new();
    src_file.read_to_string(&mut src).unwrap();

    if !check_brackets(&src) {
        panic!("Unpaired brackets");
    }

    let mut stdout = Lazy::new(|| stdout());
    let mut stdin = Lazy::new(|| stdin());

    let mut data_cursor = DataCursor::new();
    let mut inst_cursor = StringCursor::new(&src);

    loop {
        match inst_cursor.current() {
            b'<' => data_cursor.move_left(),
            b'>' => data_cursor.move_right(),
            b'+' => data_cursor.increase(),
            b'-' => data_cursor.decrease(),
            b',' => data_cursor.read_and_set(&mut *stdin),
            b'.' => data_cursor.print(&mut *stdout),
            b'[' => {
                if data_cursor.current() == 0 {
                    inst_cursor
                        .set_position(find_paired_right_bracket(&src, inst_cursor.position()));
                }
            }
            b']' => {
                if data_cursor.current() != 0 {
                    inst_cursor.set_position(find_paired_left_bracket(&src, inst_cursor.position()))
                }
            }
            _ => {}
        }
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
    for i in (position + 1)..src.len() {
        match bytes[i] {
            b'[' => {
                count += 1;
            }
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

fn check_brackets(s: &str) -> bool {
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

    fn print<W>(&self, out: &mut W)
    where
        W: Write,
    {
        out.write_1_byte(self.vec[self.pos]).unwrap();
        out.flush().unwrap();
    }

    fn read_and_set<R>(&mut self, reader: &mut R)
    where
        R: Read,
    {
        let read = reader.read_1_byte().unwrap();
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

    fn move_right(&mut self) {
        self.pos += 1;
    }

    fn move_left(&mut self) {
        self.pos -= 1;
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn set_position(&mut self, position: usize) {
        self.pos = position;
    }
}

struct Lazy<T, F>
where
    F: FnOnce() -> T + Copy,
{
    ptr: *mut *mut T,
    initializer: F,
}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T + Copy,
{
    fn new(initializer: F) -> Self {
        Self {
            ptr: Box::into_raw(Box::new(null_mut())),
            initializer,
        }
    }
}

impl<T, F> Deref for Lazy<T, F>
where
    F: FnOnce() -> T + Copy,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let obj_ptr = *self.ptr;
            if obj_ptr.is_null() {
                *self.ptr = Box::into_raw(Box::new((self.initializer)()));
            }
            &*obj_ptr
        }
    }
}

impl<T, F> DerefMut for Lazy<T, F>
where
    F: FnOnce() -> T + Copy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let obj_ptr = *self.ptr;
            if obj_ptr.is_null() {
                *self.ptr = Box::into_raw(Box::new((self.initializer)()));
            }
            &mut *obj_ptr
        }
    }
}

impl<T, F> Drop for Lazy<T, F>
where
    F: FnOnce() -> T + Copy,
{
    fn drop(&mut self) {
        unsafe {
            let obj_ptr = *self.ptr;
            drop(Box::from_raw(obj_ptr));
            drop(Box::from_raw(self.ptr as *mut *mut T));
        }
    }
}

trait ReadOneByte<R>
where
    R: Read,
{
    fn read_1_byte(&mut self) -> std::io::Result<u8>;
}

trait WriteOneByte<W>
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
