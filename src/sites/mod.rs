mod custom;
mod youtube;

use custom::Custom;
use youtube::Youtube;

use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub image: Option<String>,
    pub posts: Vec<Post>,
}

#[derive(Debug, serde::Serialize)]
pub struct Post {
    pub id: String,
    pub name: String,
    pub url: String,
    pub message: String,
    pub created_time: String,
}

pub trait Site {
    fn id(&self, url: &str) -> Option<String>;
    fn user(&self, elephantry: &elephantry::Pool, id: &str, _: &str) -> crate::Result<self::User>;

    fn fetch_html(&self, url: &str) -> crate::Result<scraper::html::Html> {
        let contents = self.fetch(attohttpc::Method::GET, url, None)?;
        let html = scraper::Html::parse_document(&contents);

        Ok(html)
    }

    fn fetch(
        &self,
        method: attohttpc::Method,
        url: &str,
        body: Option<&str>,
    ) -> crate::Result<String> {
        let http_proxy = envir::try_parse("http_proxy")?;
        let https_proxy = envir::try_parse("https_proxy")?;

        let settings = attohttpc::ProxySettingsBuilder::new()
            .http_proxy(http_proxy)
            .https_proxy(https_proxy)
            .build();

        let request = attohttpc::RequestBuilder::new(method, url)
            .proxy_settings(settings)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; rv:78.0) Gecko/20100101 Firefox/78.0",
            )
            .header("Accept-Language", "en-US")
            .header("Cache-control", "no-cache");

        let response = if let Some(body) = body {
            request
                .header("Content-Type", "application/json")
                .header("Accept", "*/*")
                .text(&body)
                .send()
        } else {
            request.send()
        }?;

        if response.status().is_success() {
            Ok(response.text()?)
        } else {
            log::error!("{:#?}", response);
            Err(crate::Error::NotFound)
        }
    }

    fn og(&self, html: &scraper::html::Html, name: &str) -> crate::Result<String> {
        let s = format!("html > head > meta[property=\"og:{name}\"]");
        let selector = scraper::Selector::parse(&s).unwrap();

        let element = match html.select(&selector).next() {
            Some(element) => element,
            None => return Err(crate::Error::NotFound),
        };

        match element.value().attr("content") {
            Some(content) => Ok(content.to_string()),
            None => Err(crate::Error::NotFound),
        }
    }

    fn select_first<'a>(
        &self,
        element: &'a scraper::ElementRef<'_>,
        selector: &'static str,
    ) -> Option<scraper::ElementRef<'a>> {
        self.select(element, selector).first().copied()
    }

    fn select<'a>(
        &self,
        element: &'a scraper::ElementRef<'_>,
        selector: &'static str,
    ) -> Vec<scraper::ElementRef<'a>> {
        static SELECTORS: std::sync::LazyLock<std::sync::Mutex<HashMap<&'static str, scraper::Selector>>> = std::sync::LazyLock::new(|| {
            std::sync::Mutex::new(HashMap::new())
        });

        let mut selectors = (*SELECTORS).lock().unwrap();

        if !selectors.contains_key(selector) {
            selectors.insert(selector, scraper::Selector::parse(selector).unwrap());
        }

        let selector = selectors.get(selector).unwrap();

        element.select(selector).collect()
    }
}

pub struct Sites {
    pub sites: HashMap<&'static str, Box<dyn Site>>,
}

impl Sites {
    pub fn new() -> Self {
        let mut sites: HashMap<&'static str, Box<dyn Site>> = HashMap::new();
        sites.insert("youtube", Box::<Youtube>::default());
        sites.insert("custom", Box::<Custom>::default());

        Self { sites }
    }

    pub fn find(&self, account: &str) -> Option<(&str, String)> {
        for (name, site) in self.sites.iter() {
            match site.id(account) {
                Some(id) => return Some((name, id)),
                None => continue,
            }
        }

        None
    }

    pub fn user(
        &self,
        elephantry: &elephantry::Pool,
        name: &str,
        id: &str,
        params: &str,
    ) -> crate::Result<User> {
        let site = match self.sites.get(name) {
            Some(site) => site,
            None => return Err(crate::Error::NotFound),
        };

        site.user(elephantry, id, params)
    }

    pub fn preview(site: &crate::site::Entity) -> crate::Result<User> {
        Custom::preview(site)
    }
}
