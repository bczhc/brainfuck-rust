use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: std::io::Error) { from() }
        UnpairedBrackets
        Infallible(err: std::num::TryFromIntError) { from() }
        InvalidUnicode
    }
}

pub type Result<T> = std::result::Result<T, Error>;
