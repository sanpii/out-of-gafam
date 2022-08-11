#[derive(Default)]
pub struct Instagram {
}

impl std::fmt::Display for Instagram
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        write!(f, "instagram")
    }
}

impl crate::sites::Site for Instagram
{
    fn id(&self, url: &str) -> Option<String>
    {
        let re = regex::Regex::new(r"https?://([^\.]+.)?instagram.com/(?P<name>[^/]+)")
            .unwrap();

        re.captures(url).map(|caps| caps["name"].to_string())
    }

    fn user(&self, _: &elephantry::Pool, id: &str, _: &str) -> crate::Result<crate::sites::User>
    {
        let url = format!("https://www.instagram.com/{}/?__a=1", id);
        let json = self.fetch_json(&url)?;

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
                json::JsonValue::String(caption) => caption.replace('\n', "<br />"),
                _ => String::new(),
            };
            let thumbnail = &edge["node"]["thumbnail_src"];
            let id = edge["node"]["shortcode"].to_string();

            let message = format!("{}<br /><img src=\"{}\" />", caption, thumbnail);

            let post = crate::sites::Post {
                name: "Post".to_string(),
                url: format!("https://www.instagram.com/p/{}", id),
                message,
                created_time: Self::parse_date(&edge["node"]["taken_at_timestamp"].to_string()),
                id,
            };

            user.posts.push(post);
        }

        Ok(user)
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
