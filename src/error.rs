pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid json response: {0}")]
    Json(#[from] json::JsonError),
    #[error("Database error: {0}")]
    Elephantry(#[from] elephantry::Error),
    #[error("Not found")]
    NotFound,
    #[error("Parse float error: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),
    #[error("Parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Unable to fetch remote resource: {0}")]
    Request(#[from] attohttpc::Error),
    #[error("Sere error{0}")]
    Serde(#[from] serde_json::Error),
    #[error("Template error: {0}")]
    Template(#[from] tera::Error),
    #[error("UTF8 decoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
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
            Error::ParseFloat(_) => StatusCode::BAD_REQUEST,
            Error::ParseInt(_) => StatusCode::BAD_REQUEST,
            Error::Request(_) => StatusCode::NOT_FOUND,
            Error::Serde(_) => StatusCode::NOT_FOUND,
            Error::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Utf8(_) => StatusCode::NOT_FOUND,
        }
    }
}

impl actix_web::error::ResponseError for Error
{
    fn error_response(&self) -> actix_web::HttpResponse
    {
        let status: actix_web::http::StatusCode = self.into();

        if status.is_client_error() {
            log::warn!("{:?}", self);
        } else if status.is_server_error() {
            log::error!("{:?}", self);
        }

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
