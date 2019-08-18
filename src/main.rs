mod error;
mod sites;

use error::Error;
use error::Result;
use sites::Sites;

struct AppState {
    sites: Sites,
    template: tera::Tera,
}

#[derive(serde_derive::Deserialize)]
struct Params {
    account: String,
}

fn main()
{
    #[cfg(debug_assertions)]
    dotenv::dotenv()
        .ok();

    let ip = std::env::var("LISTEN_IP")
        .expect("Missing LISTEN_IP env variable");
    let port = std::env::var("LISTEN_PORT")
        .expect("Missing LISTEN_IP env variable");
    let bind = format!("{}:{}", ip, port);

    actix_web::server::new(|| {
        let state = AppState {
            template: tera::compile_templates!("templates/**/*"),
            sites: Sites::new(),
        };
        let static_files = actix_web::fs::StaticFiles::new("static/")
            .expect("failed constructing static files handler");
        let errors = actix_web::middleware::ErrorHandlers::new()
                .handler(actix_web::http::StatusCode::NOT_FOUND, |req, res| error(404, req, res))
                .handler(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, |req, res| error(500, req, res));

        actix_web::App::with_state(state)
            .middleware(errors)
            .resource("/", |r| r.get().with(index))
            .resource("/search", |r| r.post().with(search))
            .resource("/show/{site}/{name:.*}", |r| r.get().f(show))
            .resource("/feed/{site}/{name:.*}", |r| r.get().f(feed))
            .resource("/show/{name:.*}", |r| r.get().f(show_fb))
            .resource("/feed/{name:.*}", |r| r.get().f(feed_fb))
            .resource("/about", |r| r.get().with(about))
            .handler("/static", static_files)
    })
    .bind(&bind)
    .expect(&format!("Can not bind to {}", bind))
    .run();
}

fn index(state: actix_web::State<AppState>) -> actix_web::HttpResponse
{
    let body = match state.template.render("index.html", &tera::Context::new()) {
        Ok(body) => body,
        Err(err) => return Error::from(err).into(),
    };

    actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn search(state: actix_web::State<AppState>, params: actix_web::Form<Params>) -> actix_web::HttpResponse
{
    if let Some((name, id)) = state.sites.find(&params.account) {
        actix_web::HttpResponse::Found()
            .header(actix_web::http::header::LOCATION, format!("/show/{}/{}", name, id))
            .finish()
    }
    else {
        actix_web::HttpResponse::NotFound()
            .finish()
    }
}

fn show_fb(request: &actix_web::HttpRequest<AppState>) -> actix_web::HttpResponse
{
    let name = &request.match_info()["name"];

    actix_web::HttpResponse::MovedPermanently()
        .header(actix_web::http::header::LOCATION, format!("/show/facebook/{}", name))
        .finish()
}

fn show(request: &actix_web::HttpRequest<AppState>) -> actix_web::HttpResponse
{
    let body = match body(request, "show.html") {
        Ok(body) => body,
        Err(err) => return err.into(),
    };

    actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn feed_fb(request: &actix_web::HttpRequest<AppState>) -> actix_web::HttpResponse
{
    let name = &request.match_info()["name"];

    actix_web::HttpResponse::MovedPermanently()
        .header(actix_web::http::header::LOCATION, format!("/feed/facebook/{}", name))
        .finish()
}

fn feed(request: &actix_web::HttpRequest<AppState>) -> actix_web::HttpResponse
{
    let body = match body(request, "rss.xml") {
        Ok(body) => body,
        Err(err) => return err.into(),
    };

    actix_web::HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .body(body)
}

fn body(request: &::actix_web::HttpRequest<AppState>, template: &str) -> Result<String>
{
    let site = &request.match_info()["site"];
    let name = &request.match_info()["name"];
    let sites = &request.state().sites;

    let group = sites.group(site, name)?;

    let mut context = tera::Context::new();
    context.insert("site", site);
    context.insert("group", &group);

    match request.state().template.render(template, &context) {
        Ok(body) => Ok(body),
        Err(err) => Err(err.into()),
    }
}

fn about(state: actix_web::State<AppState>) -> actix_web::HttpResponse
{
    let body = match state.template.render("about.html", &tera::Context::new()) {
        Ok(body) => body,
        Err(err) => return Error::from(err).into(),
    };

    actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn error(status: u32, request: &actix_web::HttpRequest<AppState>, resp: actix_web::HttpResponse)
    -> actix_web::Result<actix_web::middleware::Response>
{
    let template = format!("errors/{}.html", status);
    let body = match request.state().template.render(&template, &tera::Context::new()) {
        Ok(body) => body,
        Err(_) => "Internal server error".to_string(),
    };

    let builder = resp.into_builder()
        .header(actix_web::http::header::CONTENT_TYPE, "text/html")
        .body(body);

    Ok(actix_web::middleware::Response::Done(builder))
}
