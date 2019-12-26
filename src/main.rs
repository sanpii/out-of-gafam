mod error;
mod sites;
mod template;

use error::Error;
use error::Result;
use sites::Sites;
use template::Template;

struct AppData {
    sites: Sites,
    template: Template,
}

#[derive(serde_derive::Deserialize)]
struct Params {
    account: String,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()>
{
    env_logger::init();

    #[cfg(debug_assertions)]
    dotenv::dotenv()
        .ok();

    let ip = std::env::var("LISTEN_IP")
        .expect("Missing LISTEN_IP env variable");
    let port = std::env::var("LISTEN_PORT")
        .expect("Missing LISTEN_IP env variable");
    let bind = format!("{}:{}", ip, port);

    let template = Template::new();
    template.clone().watch();

    actix_web::HttpServer::new(move || {
        let data = AppData {
            template: template.clone(),
            sites: Sites::new(),
        };
        let static_files = actix_files::Files::new("/static", "static/");

        actix_web::App::new()
            .data(data)
            .route("/", actix_web::web::get().to(index))
            .route("/search", actix_web::web::post().to(search))
            .route("/user/{site}/{name:.*}", actix_web::web::get().to(user))
            .route("/post/{site}/{id:.*}", actix_web::web::get().to(post))
            .route("/feed/{site}/{name:.*}", actix_web::web::get().to(feed))
            .route("/about", actix_web::web::get().to(about))
            .service(static_files)
    })
    .bind(&bind)?
    .run()
    .await
}

async fn index(data: actix_web::web::Data<AppData>) -> Result<actix_web::HttpResponse>
{
    let body = match data.template.render("index.html", &tera::Context::new()) {
        Ok(body) => body,
        Err(err) => return Err(Error::from(err)),
    };

    let response = actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body);

    Ok(response)
}

fn search(data: actix_web::web::Data<AppData>, params: actix_web::web::Form<Params>) -> actix_web::HttpResponse
{
    if let Some((name, id)) = data.sites.find(&params.account) {
        actix_web::HttpResponse::Found()
            .header(actix_web::http::header::LOCATION, format!("/user/{}/{}", name, id))
            .finish()
    }
    else {
        actix_web::HttpResponse::NotFound()
            .finish()
    }
}

async fn user(request: actix_web::HttpRequest) -> Result<actix_web::HttpResponse>
{
    let body = body(&request, "user.html")?;

    let response = actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body);

    Ok(response)
}

async fn feed(request: actix_web::HttpRequest) -> Result<actix_web::HttpResponse>
{
    let body = body(&request, "rss.xml")?;

    let response = actix_web::HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .body(body);

    Ok(response)
}

fn body(request: &actix_web::HttpRequest, template: &str) -> Result<String>
{
    let site = &request.match_info()["site"];
    let name = &request.match_info()["name"];
    let data: &AppData = request.app_data()
        .unwrap();

    let user = data.sites.user(site, name)?;

    let mut context = tera::Context::new();
    context.insert("site", site);
    context.insert("user", &user);

    match data.template.render(template, &context) {
        Ok(body) => Ok(body),
        Err(err) => Err(err.into()),
    }
}

async fn about(data: actix_web::web::Data<AppData>) -> Result<actix_web::HttpResponse>
{
    let body = match data.template.render("about.html", &tera::Context::new()) {
        Ok(body) => body,
        Err(err) => return Err(Error::from(err)),
    };

    let response = actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body);

    Ok(response)
}

async fn post(request: actix_web::HttpRequest) -> Result<actix_web::HttpResponse>
{
    let site = &request.match_info()["site"];
    let name = &request.match_info()["id"];
    let data: &AppData = request.app_data()
        .unwrap();

    let post = data.sites.post(site, name)?;

    let mut context = tera::Context::new();
    context.insert("site", site);
    context.insert("post", &post);

    let body = match data.template.render("post.html", &context) {
        Ok(body) => body,
        Err(err) => return Err(err.into()),
    };

    let response = actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body);

    Ok(response)
}
