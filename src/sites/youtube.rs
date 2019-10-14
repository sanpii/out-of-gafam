pub struct Youtube {
}

impl std::fmt::Display for Youtube
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
    {
        write!(f, "youtube")
    }
}

impl Default for Youtube
{
    fn default() -> Self
    {
        Self {
        }
    }
}

impl crate::sites::Site for Youtube
{
    fn id(&self, url: &str) -> Option<String>
    {
        let re = regex::Regex::new(r"https?://([^\.]+.)?youtube.com/(?P<king>channel|user|playlist)(/|\?list=)(?P<name>[^/]+)")
            .unwrap();

        let (king, mut name) = match re.captures(url) {
            Some(caps) => (caps["king"].to_string(), caps["name"].to_string()),
            None => return None,
        };

        if &king == "user" {
            let html = match self.fetch_html(&url) {
                Ok(contents) => contents,
                Err(_) => return None,
            };

            let og_url = match self.og(&html, "url") {
                Ok(og_url) => og_url,
                Err(_) => return None,
            };

            name = match re.captures(&og_url) {
                Some(caps) => caps["name"].to_string(),
                None => return None,
            };
        }

        Some(name)
    }

    fn user(&self, id: &str) -> crate::Result<crate::sites::User>
    {
        let feed_url = if id.starts_with("PL") {
            format!("https://www.youtube.com/feeds/videos.xml?playlist_id={}", id)
        }
        else {
            format!("https://www.youtube.com/feeds/videos.xml?channel_id={}", id)
        };
        let html = self.fetch_html(&feed_url)?;

        let title_selector = scraper::Selector::parse("feed > title")
            .unwrap();

        let name = match html.select(&title_selector).nth(0) {
            Some(name) => name.inner_html(),
            None => return Err(crate::Error::NotFound),
        };

        let mut user = crate::sites::User {
            id: id.to_string(),
            name,
            description: None,
            url: format!("https://www.youtube.com/channel/{}", id),
            image: None,
            posts: vec![],
        };

        let entry_selector = scraper::Selector::parse("feed > entry")
            .unwrap();
        let title_selector = scraper::Selector::parse("title")
            .unwrap();
        let id_selector = scraper::Selector::parse("id")
            .unwrap();
        let date_selector = scraper::Selector::parse("published")
            .unwrap();

        for element in html.select(&entry_selector) {
            let name = match element.select(&title_selector).nth(0) {
                Some(name) => name.inner_html(),
                None => continue,
            };

            let id = match element.select(&id_selector).nth(0) {
                Some(id) => id.inner_html().replace("yt:video:", ""),
                None => continue,
            };

            let created_time = match element.select(&date_selector).nth(0) {
                Some(created_time) => created_time.inner_html(),
                None => Default::default(),
            };


            let gafam_url = format!("https://www.youtube.com/watch?v={}", id);

            let message = format!(r#"<iframe
        width="560"
        height="315"
        src="https://invidio.us/embed/{}"
        frameborder="0"
        allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture" allowfullscreen
        ></iframe>"#, id);

            let post = crate::sites::Post {
                name,
                gafam_url,
                url: format!("/post/youtube/{}", id),
                id,
                message,
                created_time,
            };

            user.posts.push(post);
        }

        Ok(user)
    }

    fn post(&self, id: &str) -> crate::Result<crate::sites::Post>
    {
        let gafam_url = format!("https://www.youtube.com/watch?v={}", id);
        let feed_url = format!("http://www.youtube.com/oembed?url={}&format=json", gafam_url);
        let json = self.fetch_json(&feed_url)?;

        let message = format!(r#"<iframe
    width="560"
    height="315"
    src="https://invidio.us/embed/{}"
    frameborder="0"
    allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture" allowfullscreen
    ></iframe>"#, id);

        let post = crate::sites::Post {
            name: json["title"].to_string(),
            url: format!("/post/youtube/{}", id),
            gafam_url,
            id: id.to_string(),
            message,
            created_time: Default::default(),
        };

        Ok(post)
    }
}
