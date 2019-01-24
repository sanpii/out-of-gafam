#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate tera;

mod facebook;
mod error;

use error::Error;
use error::Result;

struct AppState {
    template: ::tera::Tera,
}

#[derive(Deserialize)]
struct Params {
    account: String,
}

fn main()
{
    ::actix_web::server::new(|| {
        let template = compile_templates!("templates/**/*");
        let state = AppState { template };
        let static_files = ::actix_web::fs::StaticFiles::new("static/")
            .expect("failed constructing static files handler");
        let errors = ::actix_web::middleware::ErrorHandlers::new()
                .handler(::actix_web::http::StatusCode::NOT_FOUND, |req, res| error(404, req, res))
                .handler(::actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, |req, res| error(500, req, res));

        ::actix_web::App::with_state(state)
            .middleware(errors)
            .resource("/", |r| r.get().with(index))
            .resource("/search", |r| r.post().with(search))
            .resource("/show/{name}", |r| r.get().f(show))
            .resource("/feed/{name}", |r| r.get().f(feed))
            .resource("/about", |r| r.get().with(about))
            .handler("/static", static_files)
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run();
}

fn index(state: ::actix_web::State<AppState>) -> ::actix_web::HttpResponse
{
    let body = match state.template.render("index.html", &tera::Context::new()) {
        Ok(body) => body,
        Err(err) => return Error::from(err).into(),
    };

    ::actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn search(params: ::actix_web::Form<Params>) -> ::actix_web::HttpResponse
{
    let re = ::regex::Regex::new(r"https?://([^\.]+.)?facebook.com/(?P<name>[^/]+)")
        .unwrap();

    let name = match re.captures(&params.account) {
        Some(caps) => caps["name"].to_string(),
        None => params.account.clone(),
    };

    ::actix_web::HttpResponse::Found()
        .header(::actix_web::http::header::LOCATION, format!("/show/{}", name))
        .finish()
}

fn show(request: &::actix_web::HttpRequest<AppState>) -> ::actix_web::HttpResponse
{
    let body = match body(request, "show.html") {
        Ok(body) => body,
        Err(err) => return err.into(),
    };

    ::actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn feed(request: &::actix_web::HttpRequest<AppState>) -> ::actix_web::HttpResponse
{
    let body = match body(request, "rss.xml") {
        Ok(body) => body,
        Err(err) => return err.into(),
    };

    ::actix_web::HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .body(body)
}

fn body(request: &::actix_web::HttpRequest<AppState>, template: &str) -> Result<String>
{
    let name = &request.match_info()["name"];
    let fb = crate::facebook::Facebook::new();

    use crate::facebook::Api;
    let group = fb.group(name)?;

    let mut context = tera::Context::new();
    context.insert("group", &group);

    match request.state().template.render(template, &context) {
        Ok(body) => Ok(body),
        Err(err) => Err(err.into()),
    }
}

fn about(state: ::actix_web::State<AppState>) -> ::actix_web::HttpResponse
{
    let body = match state.template.render("about.html", &tera::Context::new()) {
        Ok(body) => body,
        Err(err) => return Error::from(err).into(),
    };

    ::actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn error(status: u32, request: &::actix_web::HttpRequest<AppState>, resp: ::actix_web::HttpResponse)
    -> ::actix_web::Result<::actix_web::middleware::Response>
{
    let template = format!("errors/{}.html", status);
    let body = match request.state().template.render(&template, &tera::Context::new()) {
        Ok(body) => body,
        Err(_) => "Internal server error".to_string(),
    };

    let builder = resp.into_builder()
        .header(::actix_web::http::header::CONTENT_TYPE, "text/html")
        .body(body);

    Ok(::actix_web::middleware::Response::Done(builder))
}
