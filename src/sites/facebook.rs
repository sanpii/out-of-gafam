#[derive(Default)]
pub struct Facebook {
}

impl crate::sites::Site for Facebook
{
    fn id(&self, url: &str) -> Option<String>
    {
        let re = regex::Regex::new(r"https?://([^\.]+.)?facebook.com/(?P<name>(groups/)?[^/]+)")
            .unwrap();

        re.captures(url).map(|caps| caps["name"].to_string())
    }

    fn user(&self, _: &elephantry::Pool, id: &str, _: &str) -> crate::Result<crate::sites::User>
    {
        let url = format!("https://mobile.facebook.com/{}", id);
        let html = self.fetch_html(&url)?;
        let root = html.root_element();

        let mut user = crate::sites::User {
            id: id.to_string(),
            name: self.og(&html, "title")
                .unwrap_or_else(|_| id.to_string()),
            description: self.og(&html, "description")
                .ok(),
            url: self.og(&html, "url")
                .unwrap_or(url),
            image: self.og(&html, "image")
                .ok(),
            posts: vec![],
        };

        let id_regex = regex::Regex::new("&id=([^&]+)")
            .unwrap();

        for element in self.select(&root, "div[data-ft]") {
            let name = match self.select_first(&element, "h3") {
                Some(e) => self.rewrite_href(&e.inner_html()),
                None => continue,
            };

            let message = match self.select_first(&element, "div > div > span") {
                Some(e) => self.rewrite_href(&e.inner_html()),
                None => continue,
            };

            let created_time = match self.select_first(&element, "abbr") {
                Some(e) => Self::parse_date(&e.inner_html()),
                None => continue,
            };

            let url = match self.select_first(&element, "div:last-child > div:last-child > a:last-child") {
                Some(e) => self.rewrite_url(e.value().attr("href").unwrap_or_default()),
                None => continue,
            };

            let id = match id_regex.captures(&url) {
                Some(caps) => caps[1].to_string(),
                None => continue,
            };

            let post = crate::sites::Post {
                name,
                url,
                message,
                created_time,
                id,
            };

            user.posts.push(post);
        }

        Ok(user)
    }
}

impl Facebook
{
    fn rewrite_href(&self, contents: &str) -> String
    {
        let regex = regex::Regex::new(r#"href="(/[^"]+)""#)
            .unwrap();

        let contents = regex.replace_all(contents, r#"href="https://mobile.facebook.com$1""#)
            .to_string();

        let regex = regex::Regex::new(r#"href="https://mobile\.facebook\.com/story\.php\?story_fbid=([^&]+)&amp;id=([^&]+)[^"]*"#)
            .unwrap();

        regex.replace_all(&contents, r#"href="/post/facebook/$1-$2"#)
            .to_string()
    }

    fn rewrite_url(&self, contents: &str) -> String
    {
        contents.replace('/', "https://mobile.facebook.com/")
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
