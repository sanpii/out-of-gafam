mod facebook;
mod instagram;
mod twitter;
mod youtube;

use facebook::Facebook;
use instagram::Instagram;
use twitter::Twitter;
use youtube::Youtube;

use std::collections::HashMap;

#[derive(Debug, serde_derive::Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub image: Option<String>,
    pub posts: Vec<Post>,
}

#[derive(Debug, serde_derive::Serialize)]
pub struct Post {
    pub id: String,
    pub name: String,
    pub url: String,
    pub gafam_url: String,
    pub message: String,
    pub created_time: String,
}

pub trait Site {
    fn id(&self, url: &str) -> Option<String>;
    fn user(&self, id: &str) -> crate::Result<self::User>;
    fn post(&self, id: &str) -> crate::Result<self::Post>;

    fn fetch_json(&self, url: &str) -> crate::Result<json::JsonValue>
    {
        let contents = self.fetch(&url)?;
        let json = json::parse(&contents)?;

        Ok(json)
    }

    fn fetch_html(&self, url: &str) -> crate::Result<scraper::html::Html>
    {
        let contents = self.fetch(&url)?;
        let html = scraper::Html::parse_document(&contents);

        Ok(html)
    }

    fn fetch(&self, url: &str) -> crate::Result<String>
    {
        let response = attohttpc::get(url)
            .header("User-Agent", "Mozilla")
            .header("Accept-Language", "en-US")
            .send()?;

        if response.status().is_success() {
            Ok(response.text()?)
        }
        else {
            Err(crate::Error::NotFound)
        }
    }

    fn og(&self, html: &scraper::html::Html, name: &str) -> crate::Result<String>
    {
        let s = format!("html > head > meta[property=\"og:{}\"]", name);
        let selector = scraper::Selector::parse(&s)
            .unwrap();

        let element = match html.select(&selector).next() {
            Some(element) => element,
            None => return Err(crate::Error::NotFound),
        };

        match element.value().attr("content") {
            Some(content) => Ok(content.to_string()),
            None => Err(crate::Error::NotFound),
        }
    }

    fn select_first<'a>(&self, element: &'a scraper::ElementRef<'_>, selector: &'static str) -> Option<scraper::ElementRef<'a>>
    {
        match self.select(element, selector).get(0) {
            Some(e) => Some(*e),
            None => None,
        }
    }

    fn select<'a>(&self, element: &'a scraper::ElementRef<'_>, selector: &'static str) -> Vec<scraper::ElementRef<'a>>
    {
        lazy_static::lazy_static! {
            static ref SELECTORS: std::sync::Mutex<std::collections::HashMap<&'static str, scraper::Selector>> =
                std::sync::Mutex::new(std::collections::HashMap::new());
        };

        let mut selectors = (*SELECTORS).lock()
            .unwrap();

        if !selectors.contains_key(selector) {
            selectors.insert(selector, scraper::Selector::parse(selector).unwrap());
        }

        let selector = selectors.get(selector)
            .unwrap();

        element.select(&selector).collect()
    }
}

pub struct Sites {
    pub sites: HashMap<&'static str, Box<dyn Site>>,
}

impl Sites
{
    pub fn new() -> Self
    {
        let mut sites: HashMap<&'static str, Box<dyn Site>> = HashMap::new();
        sites.insert("facebook", Box::new(Facebook::default()));
        sites.insert("instagram", Box::new(Instagram::default()));
        sites.insert("twitter", Box::new(Twitter::default()));
        sites.insert("youtube", Box::new(Youtube::default()));

        Self {
            sites,
        }
    }

    pub fn find(&self, account: &str) -> Option<(&str, String)>
    {
        for (name, site) in self.sites.iter() {
            match site.id(account) {
                Some(id) => return Some((name, id)),
                None => continue,
            }
        }

        None
    }

    pub fn user(&self, name: &str, id: &str) -> crate::Result<User>
    {
        let site = match self.sites.get(name) {
            Some(site) => site,
            None => return Err(crate::Error::NotFound),
        };

        site.user(id)
    }

    pub fn post(&self, name: &str, id: &str) -> crate::Result<Post>
    {
        let site = match self.sites.get(name) {
            Some(site) => site,
            None => return Err(crate::Error::NotFound),
        };

        site.post(id)
    }
}
