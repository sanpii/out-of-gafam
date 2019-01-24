pub struct Mobile {
}

impl Mobile
{
    pub fn new() -> Self
    {
        Self {
        }
    }

    fn og(html: &::scraper::html::Html, name: &str) -> crate::Result<String>
    {
        let s = format!("html > head > meta[property=\"og:{}\"]", name);
        let selector = ::scraper::Selector::parse(&s)
            .unwrap();

        let element = match html.select(&selector).nth(0) {
            Some(element) => element,
            None => return Err(crate::Error::NotFound),
        };

        match element.value().attr("content") {
            Some(content) => Ok(content.to_string()),
            None => Err(crate::Error::NotFound),
        }
    }

    fn rewrite_href(&self, contents: &str) -> String
    {
        let regex = ::regex::Regex::new(r#"href="(/[^"]+)""#)
            .unwrap();

        regex.replace_all(contents, r#"href="https://mobile.facebook.com$1""#)
            .to_string()
    }

    fn rewrite_url(&self, contents: &str) -> String
    {
        contents.replace("/", "https://mobile.facebook.com/")
    }
}

impl Mobile
{
    fn get(&self, id: &str) -> crate::Result<String>
    {
        let url = format!("https://mobile.facebook.com/{}", id);
        let client = ::reqwest::Client::new();

        let contents = client.get(&url)
            .header(::reqwest::header::USER_AGENT, "Mozilla")
            .send()?
            .text()?;

        Ok(contents)
    }
}

impl super::Api for Mobile
{
    fn group(&self, id: &str) -> crate::Result<super::Group>
    {
        let contents = self.get(id)?;
        let html = ::scraper::Html::parse_document(&contents);

        let mut group = super::Group {
            id: id.to_string(),
            name: Self::og(&html, "title")?,
            description: Self::og(&html, "description")?,
            url: Self::og(&html, "url")?,
            image: Self::og(&html, "image")?,
            posts: vec![],
        };

        let article_selector = ::scraper::Selector::parse("#recent > div > div > div[data-ft]")
            .unwrap();
        let title_selector = ::scraper::Selector::parse("h3")
            .unwrap();
        let message_selector = ::scraper::Selector::parse("div > div > span")
            .unwrap();
        let date_selector = ::scraper::Selector::parse("abbr")
            .unwrap();
        let link_selector = ::scraper::Selector::parse("div:last-child > div:last-child > a:last-child")
            .unwrap();
        let id_regex = ::regex::Regex::new("&id=([^&]+)")
            .unwrap();

        for element in html.select(&article_selector) {
            let name = match element.select(&title_selector).nth(0) {
                Some(e) => self.rewrite_href(&e.inner_html()),
                None => continue,
            };

            let message = match element.select(&message_selector).nth(0) {
                Some(e) => self.rewrite_href(&e.inner_html()),
                None => continue,
            };

            let created_time = match element.select(&date_selector).nth(0) {
                Some(e) => e.inner_html(),
                None => continue,
            };

            let permalink_url = match element.select(&link_selector).nth(0) {
                Some(e) => self.rewrite_url(e.value().attr("href").unwrap_or_default()),
                None => continue,
            };

            let id = match id_regex.captures(&permalink_url) {
                Some(caps) => caps[1].to_string(),
                None => continue,
            };

            let post = super::Post {
                name,
                permalink_url,
                message,
                created_time,
                id,
            };

            group.posts.push(post);
        }

        Ok(group)
    }
}
