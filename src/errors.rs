#[derive(Debug)]
pub struct Error(pub String);

pub type Result = std::result::Result<(), Error>;

pub fn err<T>(details: &str) -> std::result::Result<T, Error> {
    Err(Error(details.to_owned()))
}
