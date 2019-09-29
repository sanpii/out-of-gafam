#[derive(Debug)]
pub enum Error {
    Json(json::JsonError),
    NotFound,
    Request(reqwest::Error),
    Template(tera::Error),
}

impl std::fmt::Display for Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let s = match self {
            Error::Json(_) => "Invalid json response",
            Error::NotFound => "Not found",
            Error::Request(_) => "Unable to fetch remote resource",
            Error::Template(_) => "Template error",
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
            Error::Json(_) => StatusCode::NOT_FOUND,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::Request(_) => StatusCode::NOT_FOUND,
            Error::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<tera::Error> for Error
{
    fn from(err: tera::Error) -> Self
    {
        Error::Template(err)
    }
}

impl From<reqwest::Error> for Error
{
    fn from(err: reqwest::Error) -> Self
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

impl actix_web::error::ResponseError for Error
{
    fn render_response(&self) -> actix_web::HttpResponse
    {
        let status: actix_web::http::StatusCode = self.into();

        let file = format!("errors/{}.html", u16::from(status));
        let template = crate::Template::new();
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
