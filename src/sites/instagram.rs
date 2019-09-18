pub struct Instagram {
}

impl std::fmt::Display for Instagram
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
    {
        write!(f, "instagram")
    }
}

impl Default for Instagram
{
    fn default() -> Self
    {
        Self {
        }
    }
}

impl crate::sites::Site for Instagram
{
    fn id(&self, url: &str) -> Option<String>
    {
        let re = regex::Regex::new(r"https?://([^\.]+.)?instagram.com/(?P<name>[^/]+)")
            .unwrap();

        match re.captures(url) {
            Some(caps) => Some(caps["name"].to_string()),
            None => None,
        }
    }

    fn user(&self, id: &str) -> crate::Result<crate::sites::User>
    {
        let url = format!("https://www.instagram.com/{}/?__a=1", id);
        let contents = self.fetch(&url)?;
        let json = json::parse(&contents)
            .unwrap();

        let mut user = crate::sites::User {
            id: id.to_string(),
            name: json["graphql"]["user"]["username"].to_string(),
            description: Some(json["graphql"]["user"]["biography"].to_string()),
            url: format!("https://www.instagram.com/{}", id),
            image: Some(json["graphql"]["user"]["profile_pic_url"].to_string()),
            posts: vec![],
        };

        for edge in json["graphql"]["user"]["edge_owner_to_timeline_media"]["edges"].members() {
            let caption = match &edge["node"]["edge_media_to_caption"]["edges"][0]["node"]["text"] {
                json::JsonValue::String(caption) => caption.replace("\n", "<br />"),
                _ => String::new(),
            };
            let thumbnail = &edge["node"]["thumbnail_src"];

            let message = format!("{}<br /><img src=\"{}\" />", caption, thumbnail);

            let post = crate::sites::Post {
                name: "Post".to_string(),
                permalink_url: format!("/post/instagram/{}", edge["node"]["shortcode"]),
                message,
                created_time: Self::parse_date(&edge["node"]["taken_at_timestamp"].to_string()),
                id: edge["id"].to_string(),
            };

            user.posts.push(post);
        }

        Ok(user)
    }

    fn post(&self, id: &str) -> crate::Result<crate::sites::Post>
    {
        let url = format!("https://www.instagram.com/p/{}/?__a=1", id);
        let contents = self.fetch(&url)?;
        let json = json::parse(&contents)
            .unwrap();

        let caption = match &json["graphql"]["shortcode_media"]["edge_media_to_caption"]["edges"][0]["node"]["text"] {
            json::JsonValue::String(caption) => caption.replace("\n", "<br />"),
            _ => String::new(),
        };
        let thumbnail = &json["graphql"]["shortcode_media"]["display_resources"][0]["src"];

        let message = format!("{}<br /><img src=\"{}\" />", caption, thumbnail);

        let post = crate::sites::Post {
            name: "Post".to_string(),
            id: json["graphql"]["shortcode_media"]["id"].to_string(),
            permalink_url: format!("/post/instagram/{}", json["graphql"]["shortcode_media"]["shortcode"]),
            message,
            created_time: Self::parse_date(&json["graphql"]["shortcode_media"]["taken_at_timestamp"].to_string()),
        };

        Ok(post)
    }
}

impl Instagram
{
    fn parse_date(text: &str) -> String
    {
        match chrono::NaiveDateTime::parse_from_str(text, "%s") {
            Ok(date) => date.to_string(),
            Err(_) => text.to_string(),
        }
    }
}
