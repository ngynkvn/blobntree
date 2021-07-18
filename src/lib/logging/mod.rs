use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Debug)]
pub struct DisplayError<T: Display + Debug> {
    message: T,
}
impl<T: Display + Debug> From<T> for DisplayError<T> {
    fn from(message: T) -> Self {
        DisplayError { message }
    }
}
impl<T: Display + Debug> Display for DisplayError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}
impl<T: Display + Debug> Error for DisplayError<T> {}
