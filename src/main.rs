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

        ::actix_web::App::with_state(state)
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
        Err(err) => return error(err.into()),
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
        Err(err) => return error(err.into()),
    };

    ::actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn feed(request: &::actix_web::HttpRequest<AppState>) -> ::actix_web::HttpResponse
{
    let body = match body(request, "feed.xml") {
        Ok(body) => body,
        Err(err) => return error(err.into()),
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
        Err(err) => return error(err.into()),
    };

    ::actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn error(err: Error) -> ::actix_web::HttpResponse
{
    use actix_web::http::StatusCode;

    let status = match err {
        Error::NotFound => StatusCode::NOT_FOUND,
        Error::Template(err) => {
            eprintln!("{:#?}", err);

            StatusCode::INTERNAL_SERVER_ERROR
        },
    };

    ::actix_web::HttpResponse::new(status)
}
