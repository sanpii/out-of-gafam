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

    fn user(&self, id: &str) -> crate::Result<crate::sites::User>
    {
        let url = format!("https://mobile.facebook.com/{}", id);
        let contents = self.fetch(&url)?;
        let html = scraper::Html::parse_document(&contents);

        let mut user = crate::sites::User {
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

            user.posts.push(post);
        }

        Ok(user)
    }

    fn post(&self, id: &str) -> crate::Result<crate::sites::Post>
    {
        let mut t = id.split('-');
        let story_fbid = match t.nth(0) {
            Some(story_fbid) => story_fbid,
            None => return Err(crate::Error::NotFound),
        };
        let id = match t.nth(0) {
            Some(id) => id,
            None => return Err(crate::Error::NotFound),
        };

        let permalink_url = format!("https://mobile.facebook.com/story.php?story_fbid={}&id={}", story_fbid, id);
        let contents = self.fetch(&permalink_url)?;
        let html = scraper::Html::parse_document(&contents);

        let story_selector = scraper::Selector::parse("#m_story_permalink_view")
            .unwrap();
        let title_selector = scraper::Selector::parse("#m_story_permalink_view h3 > span > strong > a")
            .unwrap();
        let date_selector = scraper::Selector::parse("abbr")
            .unwrap();

        let story = match html.select(&story_selector).nth(0) {
            Some(story) => story,
            None => return Err(crate::Error::NotFound),
        };

        let title = match html.select(&title_selector).nth(0) {
            Some(title) => title,
            None => return Err(crate::Error::NotFound),
        };

        let created_time = match story.select(&date_selector).nth(0) {
            Some(e) => Self::parse_date(&e.inner_html()),
            None => Default::default(),
        };

        let post = crate::sites::Post {
            name: title.inner_html(),
            id: id.to_string(),
            permalink_url,
            message: story.inner_html(),
            created_time,
        };

        Ok(post)
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

        let contents = regex.replace_all(&contents, r#"href="https://mobile.facebook.com$1""#)
            .to_string();

        let regex = regex::Regex::new(r#"href="https://mobile\.facebook\.com/story\.php\?story_fbid=([^&]+)&amp;id=([^&]+)[^"]*"#)
            .unwrap();

        regex.replace_all(&contents, r#"href="/post/facebook/$1-$2"#)
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
