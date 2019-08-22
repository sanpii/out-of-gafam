pub struct Facebook {
}

impl Default for Facebook
{
    fn default() -> Self
    {
        Self {
        }
    }
}

impl crate::sites::Site for Facebook
{
    fn id(&self, url: &str) -> Option<String>
    {
        let re = regex::Regex::new(r"https?://([^\.]+.)?facebook.com/(?P<name>(groups/)?[^/]+)")
            .unwrap();

        match re.captures(url) {
            Some(caps) => Some(caps["name"].to_string()),
            None => None,
        }
    }

    fn group(&self, id: &str) -> crate::Result<crate::sites::Group>
    {
        let url = format!("https://mobile.facebook.com/{}", id);
        let contents = self.fetch(&url)?;
        let html = scraper::Html::parse_document(&contents);

        let mut group = crate::sites::Group {
            id: id.to_string(),
            name: Self::og(&html, "title")
                .unwrap_or_else(|_| id.to_string()),
            description: Self::og(&html, "description")
                .ok(),
            url: Self::og(&html, "url")
                .unwrap_or(url),
            image: Self::og(&html, "image")
                .ok(),
            posts: vec![],
        };

        let article_selector = scraper::Selector::parse("div[data-ft]")
            .unwrap();
        let title_selector = scraper::Selector::parse("h3")
            .unwrap();
        let message_selector = scraper::Selector::parse("div > div > span")
            .unwrap();
        let date_selector = scraper::Selector::parse("abbr")
            .unwrap();
        let link_selector = scraper::Selector::parse("div:last-child > div:last-child > a:last-child")
            .unwrap();
        let id_regex = regex::Regex::new("&id=([^&]+)")
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
                Some(e) => Self::parse_date(&e.inner_html()),
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

            let post = crate::sites::Post {
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

impl Facebook
{
    fn og(html: &scraper::html::Html, name: &str) -> crate::Result<String>
    {
        let s = format!("html > head > meta[property=\"og:{}\"]", name);
        let selector = scraper::Selector::parse(&s)
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
        let regex = regex::Regex::new(r#"href="(/[^"]+)""#)
            .unwrap();

        regex.replace_all(contents, r#"href="https://mobile.facebook.com$1""#)
            .to_string()
    }

    fn rewrite_url(&self, contents: &str) -> String
    {
        contents.replace("/", "https://mobile.facebook.com/")
    }

    fn parse_date(text: &str) -> String
    {
        let regex = regex::Regex::new("^(\\d+) hrs$")
            .unwrap();

        let relative_time = regex.replace(text, "-$1 hours");

        match chrono_english::parse_date_string(
            &relative_time,
            chrono::Local::now(),
            chrono_english::Dialect::Uk
        ) {
            Ok(date) => date.to_string(),
            Err(_) => relative_time.to_string(),
        }
    }
}
