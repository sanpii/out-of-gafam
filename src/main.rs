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
    let body = body(request, "show.html");

    ::actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn feed(request: &::actix_web::HttpRequest<AppState>) -> impl ::actix_web::Responder
{
    let body = body(request, "feed.xml");

    ::actix_web::HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .body(body)
}

fn body(request: &::actix_web::HttpRequest<AppState>, template: &str) -> String
{
    let name = &request.match_info()["name"];
    let fb = crate::facebook::Facebook::new();

    let mut context = tera::Context::new();
    context.insert("name", &name);
    context.insert("group", &fb.group(name));

    request.state().template
        .render(template, &context)
        .unwrap()
}

fn about(state: ::actix_web::State<AppState>) -> impl ::actix_web::Responder
{
    let body = state.template
        .render("about.html", &tera::Context::new())
        .unwrap();

    ::actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}
