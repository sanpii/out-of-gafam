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

impl Into<::actix_web::HttpResponse> for Error
{
    fn into(self) -> ::actix_web::HttpResponse
    {
        let status = match self {
            Error::NotFound => ::actix_web::http::StatusCode::NOT_FOUND,
            Error::Template(err) => {
                eprintln!("{:?}", err);

                ::actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            },
        };

        ::actix_web::HttpResponse::new(status)
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
