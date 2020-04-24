pub struct Twitter {
}

impl Default for Twitter
{
    fn default() -> Self
    {
        Self {
        }
    }
}

impl crate::sites::Site for Twitter
{
    fn id(&self, url: &str) -> Option<String>
    {
        let re = regex::Regex::new(r"https?://([^\.]+.)?twitter.com/search\?q=(?P<search>[^&]+)")
            .unwrap();

        if let Some(caps) = re.captures(url) {
            return Some(caps["search"].to_string());
        }

        let re = regex::Regex::new(r"https?://([^\.]+.)?twitter.com/(?P<name>[^/]+)")
            .unwrap();

        match re.captures(url) {
            Some(caps) => Some(format!("@{}", caps["name"].to_string())),
            None => None,
        }
    }

    fn user(&self, id: &str) -> crate::Result<crate::sites::User>
    {
        let (url, gafam_url) = if id.starts_with('@') {
            (
                format!("https://mobile.twitter.com/{}", id),
                format!("https://twitter.com/{}", id),
            )
        }
        else {
            let id = urlencoding::encode(id);

            (
                format!("https://mobile.twitter.com/search?q={}", id),
                format!("https://twitter.com/search?q={}", id),
            )
        };

        let html = self.fetch_html(&url)?;
        let root = html.root_element();

        let mut user = crate::sites::User {
            id: id.to_string(),
            name: self.og(&html, "title")
                .unwrap_or_else(|_| id.to_string()),
            description: None,
            url: gafam_url,
            image: if id.starts_with('@') {
                self.select_first(&root, "img[src^=\"https://pbs.twimg.com/profile_images/\"]")
                    .and_then(|e| e.value().attr("src").map(|e| e.to_string()))
            } else {
                None
            },
            posts: vec![],
        };

        for element in self.select(&root, ".tweet") {
            let name = format!("tweet de {}", id);

            let (id, message) = match self.select_first(&element, ".tweet-text") {
                Some(e) => (e.value().attr("data-id").unwrap().to_string(), e.inner_html()),
                None => continue,
            };

            let created_time = match self.select_first(&element, ".timestamp a") {
                Some(e) => Self::parse_date(&e.inner_html()),
                None => continue,
            };

            let url = format!("/post/twitter/{}", id);

            let gafam_url = match element.value().attr("href") {
                Some(gafam_url) => format!("https://twitter.com{}", gafam_url),
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
        let url = format!("/post/twitter/{}", id);

        let gafam_url = format!("https://mobile.twitter.com/oog/status/{}", id);
        let html = self.fetch_html(&gafam_url)?;
        let root = html.root_element();

        let name = match self.select_first(&root, ".username") {
            Some(e) => format!("tweet de {}", e.text().collect::<Vec<_>>().join("")),
            None => return Err(crate::Error::NotFound),
        };

        let message = match self.select_first(&root, ".tweet-text") {
            Some(e) => e.inner_html(),
            None => return Err(crate::Error::NotFound),
        };

        let created_time = match self.select_first(&root, ".tweet-content .metadata a") {
            Some(e) => Self::parse_date(&e.inner_html()),
            None => return Err(crate::Error::NotFound),
        };

        let post = crate::sites::Post {
            name,
            id: id.to_string(),
            url,
            gafam_url,
            message,
            created_time,
        };

        Ok(post)
    }
}

impl Twitter
{
    fn parse_date(text: &str) -> String
    {
        match chrono_english::parse_date_string(
            text,
            chrono::Local::now(),
            chrono_english::Dialect::Uk
        ) {
            Ok(date) => date.to_string(),
            Err(_) => text.to_string(),
        }
    }
}
