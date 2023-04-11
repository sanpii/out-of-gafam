#[derive(Default)]
pub struct Leboncoin;

impl super::Site for Leboncoin
{
    fn id(&self, url: &str) -> Option<String>
    {
        let re = regex::Regex::new(r"https?://www\.leboncoin\.fr/recherche/?(?P<param>\?.*)")
            .unwrap();

        let param = match re.captures(url) {
            Some(caps) => caps["param"].to_string(),
            None => return None,
        };

        Some(param)
    }

    fn user(&self, _: &elephantry::Pool, _: &str, params: &str) -> crate::error::Result<super::User>
    {
        let body = self.query(params)?;
        let json = self.post_json("https://api.leboncoin.fr/finder/search", &body.to_string())?;

        let mut user = crate::sites::User {
            id: "recherche".to_string(),
            name: "Recherche leboncoin".to_string(),
            description: None,
            url: format!("https://www.leboncoin.fr/recherche?{}", params),
            image: None,
            posts: vec![],
        };

        if let serde_json::Value::Array(ads) = &json["ads"] {
            for ad in ads {
                let post = crate::sites::Post {
                    created_time: ad["index_date"].to_string(),
                    url: ad["url"].to_string(),
                    id: ad["list_id"].to_string(),
                    message: ad["body"].to_string(),
                    name: ad["subject"].to_string(),
                };

                user.posts.push(post);
            }
        }

        Ok(user)
    }
}

impl Leboncoin {
    fn query(&self, params: &str) -> crate::error::Result<serde_json::Value> {
        let mut body = serde_json::json!({
            "limit": 35,
            "limit_alu": 3,
            "filters": {
                "location": {
                },
                "category" : {
                }
            }
        });

        let filters = &mut body["filters"];

        for (key, value) in form_urlencoded::parse(params.as_bytes()) {
            match key.into_owned().as_str() {
                "category" => filters["category"]["id"] = value.into_owned().into(),
                "text" => filters["keywords"]["text"] = value.into_owned().into(),
                "locations" => filters["location"]["locations"] = self.locations(value.into_owned())?,
                "price" => filters["ranges"]["price"] = self.price(value.into_owned())?,
                k => log::warn!("Unsuported query filter: {}", k),
            }
        }

        Ok(body)
    }

    fn locations(&self, param: String) -> crate::error::Result<serde_json::Value> {
        let mut locations = Vec::new();

        for l in param.split(',') {
            locations.push(self.location(l)?);
        }

        Ok(locations.into())
    }

    fn location(&self, param: &str) -> crate::error::Result<serde_json::Value> {
        let tokens = param.split('_')
            .collect::<Vec<_>>();

        let city = tokens[0];
        let lat: f32 = tokens[2].parse()?;
        let lng: f32 = tokens[3].parse()?;
        let default_radius: u32 = tokens[4].parse()?;
        let radius: u32 = tokens.get(5).unwrap_or(&"50000").parse()?;

        let json = serde_json::json!({
            "area": {
                "default_radius": default_radius,
                "lat": lat,
                "lng": lng,
                "radius": radius,
            },
            "city": city,
            "locationType": "city"
        });

        Ok(json)
    }

    fn price(&self, param: String) -> crate::error::Result<serde_json::Value> {
        let tokens = param.split('-')
            .collect::<Vec<_>>();

        let mut object = serde_json::json!({});

        if tokens[0] != "min" {
            let min: f32 = tokens[0].parse()?;
            object["min"] = min.into();
        }
        if tokens[1] != "max" {
            let max: f32 = tokens[1].parse()?;
            object["max"] = max.into();
        }

        Ok(object.into())
    }
}
