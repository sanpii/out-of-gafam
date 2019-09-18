mod facebook;
mod instagram;

use facebook::Facebook;
use instagram::Instagram;

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
    pub permalink_url: String,
    pub message: String,
    pub created_time: String,
}

pub trait Site {
    fn id(&self, url: &str) -> Option<String>;
    fn user(&self, id: &str) -> crate::Result<self::User>;
    fn post(&self, id: &str) -> crate::Result<self::Post>;

    fn fetch(&self, url: &str) -> crate::Result<String>
    {
        let client = reqwest::Client::new();

        let mut response = client.get(url)
            .header(reqwest::header::USER_AGENT, "Mozilla")
            .header(reqwest::header::ACCEPT_LANGUAGE, "en-US")
            .send()?;

        if response.status().is_success() {
            Ok(response.text()?)
        }
        else {
            Err(crate::Error::NotFound)
        }
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
