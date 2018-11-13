#[derive(Debug)]
pub struct Error(String);

pub type Result = std::result::Result<(), Error>;

pub fn err(details: &str) -> Result {
    Err(Error(details.to_owned()))
}
