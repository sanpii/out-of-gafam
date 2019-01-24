#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate tera;

mod facebook;

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
            .resource("/", |r| r.with(index))
            .resource("/search", |r| r.with(search))
            .resource("/show/{name}", |r| r.f(show))
            .resource("/feed/{name}", |r| r.f(feed))
            .handler("/static", static_files)
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run();
}

fn index(state: ::actix_web::State<AppState>) -> impl ::actix_web::Responder
{
    let body = state.template
        .render("index.html", &tera::Context::new())
        .unwrap();

    ::actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn search(params: ::actix_web::Form<Params>) -> impl ::actix_web::Responder
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

fn show(request: &::actix_web::HttpRequest<AppState>) -> impl ::actix_web::Responder
{
    let name = &request.match_info()["name"];
    let fb = crate::facebook::Facebook::new();

    let mut context = tera::Context::new();
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
    let fb = crate::facebook::Facebook::new();

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
