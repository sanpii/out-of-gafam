#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(json::JsonError),
    Elephantry(elephantry::Error),
    NotFound,
    Request(attohttpc::Error),
    Serde(serde_json::Error),
    Template(tera::Error),
    Url(urlencoding::FromUrlEncodingError),
}

impl std::fmt::Display for Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let s = match self {
            Error::Io(_) => "I/O error",
            Error::Json(_) => "Invalid json response",
            Error::Elephantry(_) => "Database error",
            Error::NotFound => "Not found",
            Error::Request(_) => "Unable to fetch remote resource",
            Error::Serde(_) => "Serede error",
            Error::Template(_) => "Template error",
            Error::Url(_) => "URL decoding error",
        };

        write!(f, "{}", s)
    }
}

impl Into<actix_web::http::StatusCode> for &Error
{
    fn into(self) -> actix_web::http::StatusCode
    {
        use actix_web::http::StatusCode;

        match self {
            Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Json(_) => StatusCode::NOT_FOUND,
            Error::Elephantry(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::Request(_) => StatusCode::NOT_FOUND,
            Error::Serde(_) => StatusCode::NOT_FOUND,
            Error::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Url(_) => StatusCode::NOT_FOUND,
        }
    }
}

impl From<elephantry::Error> for Error
{
    fn from(err: elephantry::Error) -> Self
    {
        Error::Elephantry(err)
    }
}

impl From<std::io::Error> for Error
{
    fn from(err: std::io::Error) -> Self
    {
        Error::Io(err)
    }
}

impl From<tera::Error> for Error
{
    fn from(err: tera::Error) -> Self
    {
        Error::Template(err)
    }
}

impl From<attohttpc::Error> for Error
{
    fn from(err: attohttpc::Error) -> Self
    {
        Error::Request(err)
    }
}

impl From<json::JsonError> for Error
{
    fn from(err: json::JsonError) -> Self
    {
        Error::Json(err)
    }
}

impl From<urlencoding::FromUrlEncodingError> for Error
{
    fn from(err: urlencoding::FromUrlEncodingError) -> Self
    {
        Error::Url(err)
    }
}

impl From<serde_json::Error> for Error
{
    fn from(err: serde_json::Error) -> Self
    {
        Error::Serde(err)
    }
}

impl actix_web::error::ResponseError for Error
{
    fn error_response(&self) -> actix_web::HttpResponse
    {
        let status: actix_web::http::StatusCode = self.into();

        let file = format!("errors/{}.html", u16::from(status));
        let template = tera_hot::Template::new(crate::TEMPLATE_DIR);
        let body = match template.render(&file, &tera::Context::new()) {
            Ok(body) => body,
            Err(err) => {
                eprintln!("{:?}", err);

                "Internal server error".to_string()
            }
        };

        actix_web::HttpResponse::build(status)
            .header(actix_web::http::header::CONTENT_TYPE, "text/html")
            .body(body)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
