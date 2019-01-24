#[derive(Debug)]
pub enum Error {
    NotFound,
    Template(::tera::Error),
}

impl From<::tera::Error> for Error
{
    fn from(err: ::tera::Error) -> Self
    {
        Error::Template(err)
    }
}

impl From<::reqwest::Error> for Error
{
    fn from(_: ::reqwest::Error) -> Self
    {
        Error::NotFound
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
