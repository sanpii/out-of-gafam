#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate tera;

mod facebook;

struct AppState {
    template: ::tera::Tera,
}

fn main()
{
    ::actix_web::server::new(|| {
        let template = compile_templates!("templates/**/*");
        let state = AppState { template };
        let static_files = ::actix_web::fs::StaticFiles::new("static/")
            .expect("failed constructing static files handler");

        ::actix_web::App::with_state(state)
            .resource("/", |r| r.f(index))
            .resource("/show/{name}", |r| r.f(show))
            .resource("/feed/{name}", |r| r.f(feed))
            .resource("/privacy", |r| r.f(privacy))
            .handler("/static", static_files)
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run();
}

fn index(request: &::actix_web::HttpRequest<AppState>) -> impl ::actix_web::Responder
{
    match request.method().as_str() {
        "GET" => {
            let body = request.state().template
                .render("index.html", &tera::Context::new())
                .unwrap();

            ::actix_web::HttpResponse::Ok()
                .content_type("text/html")
                .body(body)
        },
        "POST" => {
            let re = ::regex::Regex::new(r"https?://www.facebook.com/(?P<name>[^/]+)")
                .unwrap();
            let caps = re.captures("https://www.facebook.com/streetmedicsnantes")
                .unwrap();

            ::actix_web::HttpResponse::Found()
                .header(::actix_web::http::header::LOCATION, format!("/show/{}", &caps["name"]))
                .finish()
        },
        _ => ::actix_web::HttpResponse::BadRequest()
            .finish(),
    }
}

fn show(request: &::actix_web::HttpRequest<AppState>) -> impl ::actix_web::Responder
{
    let name = &request.match_info()["name"];
    let fb = crate::facebook::Facebook::new("token");

    let mut context = tera::Context::new();
    context.insert("name", &name);
    context.insert("group", &fb.group(name));

    let body = request.state().template
        .render("show.html", &context)
        .unwrap();

    ::actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn feed(request: &::actix_web::HttpRequest<AppState>) -> impl ::actix_web::Responder
{
    let name = &request.match_info()["name"];
    let fb = crate::facebook::Facebook::new("token");

    let mut context = tera::Context::new();
    context.insert("name", &name);
    context.insert("group", &fb.group(name));

    let body = request.state().template
        .render("rss.xml", &context)
        .unwrap();

    ::actix_web::HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .body(body)
}

fn privacy(request: &::actix_web::HttpRequest<AppState>) -> impl ::actix_web::Responder
{
    let body = request.state().template
        .render("privacy.html", &tera::Context::new())
        .unwrap();

    ::actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}
