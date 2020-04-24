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

            let url = format!("/post/facebook/{}", id);

            let gafam_url = match self.select_first(&element, "div:last-child > div:last-child > a:last-child") {
                Some(e) => self.rewrite_url(e.value().attr("href").unwrap_or_default()),
                None => continue,
            };

            let id = match id_regex.captures(&gafam_url) {
                Some(caps) => caps[1].to_string(),
                None => continue,
            };

            let post = crate::sites::Post {
                name,
                url,
                gafam_url,
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
        let url = format!("/post/facebook/{}", id);
        let mut t = id.split('-');
        let story_fbid = match t.next() {
            Some(story_fbid) => story_fbid,
            None => return Err(crate::Error::NotFound),
        };
        let id = match t.next() {
            Some(id) => id,
            None => return Err(crate::Error::NotFound),
        };

        let gafam_url = format!("https://mobile.facebook.com/story.php?story_fbid={}&id={}", story_fbid, id);
        let html = self.fetch_html(&gafam_url)?;
        let root = html.root_element();

        let story = match self.select_first(&root, "#m_story_permalink_view") {
            Some(story) => story,
            None => return Err(crate::Error::NotFound),
        };

        let title = match self.select_first(&root, "#m_story_permalink_view h3 > span > strong > a") {
            Some(title) => title,
            None => return Err(crate::Error::NotFound),
        };

        let created_time = match self.select_first(&story, "abbr") {
            Some(e) => Self::parse_date(&e.inner_html()),
            None => Default::default(),
        };

        let post = crate::sites::Post {
            name: title.inner_html(),
            id: id.to_string(),
            url,
            gafam_url,
            message: story.inner_html(),
            created_time,
        };

        Ok(post)
    }
}

impl Facebook
{
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
