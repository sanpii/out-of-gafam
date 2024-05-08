#![warn(rust_2018_idioms)]

mod error;
mod site;
mod sites;

use error::Error;
use error::Result;
use sites::Sites;

struct AppData {
    sites: Sites,
    template: tera_hot::Template,
    elephantry: elephantry::Pool,
}

#[derive(serde::Deserialize)]
struct Params {
    account: String,
}

static TEMPLATE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/templates");

#[actix_web::main]
async fn main() -> Result {
    #[cfg(debug_assertions)]
    envir::dotenv();

    env_logger::init();

    let database_url = envir::get("DATABASE_URL")?;
    let ip = envir::get("LISTEN_IP")?;
    let port = envir::get("LISTEN_PORT")?;
    let bind = format!("{}:{}", ip, port);

    let template = tera_hot::Template::new(TEMPLATE_DIR);
    template.clone().watch();

    let elephantry = elephantry::Pool::new(&database_url).expect("Unable to connect to postgresql");

    actix_web::HttpServer::new(move || {
        let data = AppData {
            elephantry: elephantry.clone(),
            template: template.clone(),
            sites: Sites::new(),
        };
        let static_files = actix_files::Files::new("/static", "static/");

        actix_web::App::new()
            .app_data(data)
            .route("/", actix_web::web::get().to(index))
            .route("/search", actix_web::web::post().to(search))
            .route("/preview", actix_web::web::get().to(preview))
            .route("/preview", actix_web::web::post().to(save))
            .route("/iframe", actix_web::web::get().to(iframe))
            .route("/user/{site}/{name:.*}", actix_web::web::get().to(user))
            .route("/feed/{site}/{name:.*}", actix_web::web::get().to(feed))
            .route("/about", actix_web::web::get().to(about))
            .service(static_files)
    })
    .bind(&bind)?
    .run()
    .await?;

    Ok(())
}

async fn index(request: actix_web::HttpRequest) -> Result<actix_web::HttpResponse> {
    let data: &AppData = request.app_data().unwrap();
    let body = data.template.render("index.html", &tera::Context::new())?;

    let response = actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body);

    Ok(response)
}

async fn search(
    request: actix_web::HttpRequest,
    params: actix_web::web::Form<Params>,
) -> Result<actix_web::HttpResponse> {
    let data: &AppData = request.app_data().unwrap();

    let url = if let Some((name, id)) = data.sites.find(&params.account) {
        format!("/user/{}/{}", name, id)
    } else {
        match data
            .elephantry
            .model::<crate::site::Model>()
            .find(&params.account)?
        {
            Some(site) => format!("/user/custom/{}", site.id.unwrap().as_hyphenated()),
            None => format!(
                "/preview?channel_link={}",
                urlencoding::encode(&params.account)
            ),
        }
    };

    let response = actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, url))
        .finish();

    Ok(response)
}

async fn user(request: actix_web::HttpRequest) -> Result<actix_web::HttpResponse> {
    let body = body(&request, "user.html")?;

    let response = actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body);

    Ok(response)
}

async fn feed(request: actix_web::HttpRequest) -> Result<actix_web::HttpResponse> {
    let body = body(&request, "rss.xml")?;

    let response = actix_web::HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .body(body);

    Ok(response)
}

fn body(request: &actix_web::HttpRequest, template: &str) -> Result<String> {
    let site = &request.match_info()["site"];
    let name = &request.match_info()["name"];
    let params = request.query_string();
    let data: &AppData = request.app_data().unwrap();

    let user = data.sites.user(&data.elephantry, site, name, params)?;

    let mut context = tera::Context::new();
    context.insert("site", site);
    context.insert("params", &params);
    context.insert("user", &user);

    match data.template.render(template, &context) {
        Ok(body) => Ok(body),
        Err(err) => Err(err.into()),
    }
}

async fn about(request: actix_web::HttpRequest) -> Result<actix_web::HttpResponse> {
    let data: &AppData = request.app_data().unwrap();
    let body = data.template.render("about.html", &tera::Context::new())?;

    let response = actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body);

    Ok(response)
}

async fn iframe(
    query: actix_web::web::Query<std::collections::HashMap<String, String>>,
) -> Result<actix_web::HttpResponse> {
    let url = query.get("url").unwrap();

    let body = attohttpc::get(urlencoding::decode(url)?)
        .header("User-Agent", "Mozilla")
        .header("Accept-Language", "en-US")
        .send()?;

    let response = actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body.text()?);

    Ok(response)
}

async fn preview(
    request: actix_web::HttpRequest,
    site: actix_web::web::Query<site::Entity>,
) -> Result<actix_web::HttpResponse> {
    let data: &AppData = request.app_data().unwrap();

    let mut context = tera::Context::new();
    context.insert("data", &*site);
    context.insert("user", &Sites::preview(&site)?);
    context.insert("iframe", &format!("/iframe?url={}", site.channel_link));

    let body = data.template.render("preview.html", &context)?;

    let response = actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body);

    Ok(response)
}

async fn save(
    request: actix_web::HttpRequest,
    site: actix_web::web::Query<site::Entity>,
) -> Result<actix_web::HttpResponse> {
    let data: &AppData = request.app_data().unwrap();

    let site = match data.elephantry.insert_one::<site::Model>(&site) {
        Ok(site) => site,
        Err(elephantry::Error::Sql(err)) => {
            if err.state()? == Some(elephantry::pq::state::UNIQUE_VIOLATION) {
                data.elephantry
                    .model::<site::Model>()
                    .find(&site.channel_link)?
                    .unwrap()
            } else {
                return Err(elephantry::Error::Sql(err).into());
            }
        }
        Err(err) => return Err(err.into()),
    };

    let url = match site.id {
        Some(id) => format!("/user/custom/{}", id),
        None => return Err(Error::NotFound),
    };

    let response = actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, url))
        .finish();

    Ok(response)
}
