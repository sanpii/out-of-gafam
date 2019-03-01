#[cfg(feature = "no-api")]
mod mobile;
#[cfg(not(feature = "no-api"))]
mod graph;

#[derive(Debug, serde_derive::Serialize)]
pub struct Group {
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

pub trait Api {
    fn group(&self, name: &str) -> crate::Result<self::Group>;
}

pub struct Facebook {
    api: Box<Api>,
}

impl Facebook
{
    pub fn new() -> Self
    {
        Facebook {
            api: Box::new(Self::api()),
        }
    }

    #[cfg(feature = "no-api")]
    fn api() -> impl Api
    {
        self::mobile::Mobile::new()
    }

    #[cfg(not(feature = "no-api"))]
    fn api() -> impl Api
    {
        self::graph::Graph::new("token")
    }
}

impl Api for Facebook
{
    fn group(&self, name: &str) -> crate::Result<self::Group>
    {
        self.api.group(name)
    }
}
