pub struct Graph {
    token: String,
}

impl Graph
{
    pub fn new(token: &str) -> Self
    {
        Self {
            token: token.to_string(),
        }
    }
}

impl super::Api for Graph
{
    fn group(&self, name: &str) -> super::Group
    {
        super::Group {
            id: name.to_string(),
            name: name.to_string(),
            description: name.to_string(),
            image: String::new(),
            url: format!("https://mobile.facebook.com/{}", name),
            posts: vec![
                super::Post {
                    name: "Assemblée des blessé-e-s 44's".to_string(),
                    permalink_url: "https://www.facebook.com/Assemblée-des-blessé-e-s-44-1751594901723011/".to_string(),
                    message: "6 blessé.es dans les manifestations portent plainte pour violence policière dont Adrien (fracture du crâne et hémorragie cérébrale) et Philippe (rupture de la rate et hémorragie interne). Ne laissons pas passer les violences policières !".to_string(),
                    created_time: "January 11 at 5:06 AM".to_string(),
                    id: "703".to_string(),
                }
            ],
        }
    }
}
