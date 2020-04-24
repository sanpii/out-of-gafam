pub struct Custom {
}

impl std::fmt::Display for Custom
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        write!(f, "custom")
    }
}

impl Default for Custom
{
    fn default() -> Self
    {
        Self {
        }
    }
}

impl crate::sites::Site for Custom
{
    fn id(&self, _: &str) -> Option<String>
    {
        None
    }

    fn user(&self, elephantry: &elephantry::Pool, id: &str) -> crate::Result<crate::sites::User>
    {
        let uuid = uuid::Uuid::parse_str(id).unwrap();
        let entity = match elephantry.find_by_pk::<crate::site::Model<'_>>(&elephantry::pk! {
            id => uuid,
        })? {
            Some(entity) => entity,
            None => return Err(crate::Error::NotFound),
        };

        Self::preview(&entity)
    }

    fn post(&self, _: &str) -> crate::Result<crate::sites::Post>
    {
        Err(crate::Error::NotFound)
    }
}

impl Custom
{
    pub fn preview(data: &crate::site::Entity) -> crate::Result<crate::sites::User> {
        let url = urlencoding::decode(&data.channel_link)?;
        let body = attohttpc::get(&url)
            .header("User-Agent", "Mozilla")
            .header("Accept-Language", "en-US")
            .send()?;
        let html = scraper::Html::parse_document(&body.text()?);
        let root = html.root_element();

        let mut posts = Vec::new();

        for element in Self::select(&root, &data.items) {
            posts.push(crate::sites::Post {
                id: String::new(),
                name: Self::get_one(&element, Some(&data.item_title)).unwrap_or_else(|| data.item_title.clone()),
                url: Self::get_one(&element, Some(&data.item_link)).unwrap_or_else(|| data.item_link.clone()),
                gafam_url: String::new(),
                message: Self::get_one(&element, Some(&data.item_description)).unwrap_or_else(|| data.item_description.clone()),
                created_time: Self::get_one(&element, Some(&data.item_pubdate)).unwrap_or_else(|| data.item_pubdate.clone()),
            });
        }

        let user = crate::sites::User {
            url,
            id: data.id.map(|x| x.to_hyphenated().to_string()).unwrap_or_default(),
            description: Self::get_one(&root, data.channel_description.as_ref()),
            name: Self::get_one(&root, Some(&data.channel_title)).unwrap_or_default(),
            image: Self::get_one(&root, data.channel_image.as_ref()),
            posts,
        };

        Ok(user)
    }

    fn get_one<'a>(root: &'a scraper::ElementRef<'_>, selector: Option<&String>) -> Option<String>
    {
        if let Some(selector) = selector {
            Self::get_all(root, selector)
                .get(0)
                .cloned()
        } else {
            None
        }
    }

    fn get_all<'a>(root: &'a scraper::ElementRef<'_>, selector: &str) -> Vec<String>
    {
        let (selector, attr) = Self::parse_selector(selector);

        Self::select(root, &selector)
            .iter()
            .map(|x| if let Some(attr) = &attr {
                x.value().attr(&attr).unwrap_or_default().to_string()
            } else {
                x.inner_html()
            })
        .collect()
    }

    fn select<'a>(root: &'a scraper::ElementRef<'_>, selector: &str) -> Vec<scraper::ElementRef<'a>>
    {
        if selector.is_empty() {
            return Vec::new();
        }

        if selector == "." {
            return vec![*root];
        }

        match scraper::Selector::parse(&selector) {
            Ok(selector) => root.select(&selector).collect(),
            Err(_) => Vec::new(),
        }
    }

    fn parse_selector(x: &str) -> (String, Option<String>)
    {
        if !x.contains('[') {
            return (x.to_string(), None);
        }

        let re = regex::Regex::new(r"(?P<selector>.+)(\[(?P<attr>[^\]]+)\])$")
            .unwrap();

        let (mut selector, mut attr) = match re.captures(x) {
            Some(caps) => (caps["selector"].to_string(), caps.name("attr").map(|x| x.as_str().to_string())),
            None => (x.to_string(), None),
        };

        if let Some(x) = &attr {
            if x.contains('=') {
                selector = format!("{}[{}]", selector, x);
                attr = None;
            }
        }

        (selector, attr)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn parse_selector() {
        let patterns = vec![
            ("main h1 > span", ("main h1 > span".to_string(), None)),
            ("html > head > meta[property=\"og:icon\"]", ("html > head > meta[property=\"og:icon\"]".to_string(), None)),
            (".field__item li > strong > a[href]", (".field__item li > strong > a".to_string(), Some("href".to_string()))),
            ("html > head > meta[property=\"og:icon\"][content]", ("html > head > meta[property=\"og:icon\"]".to_string(), Some("content".to_string()))),
        ];

        for (pattern, expected) in &patterns {
            assert_eq!(&crate::sites::Custom::parse_selector(pattern), expected);
        }
    }
}
