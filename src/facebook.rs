pub struct Facebook {
    token: String,
}

#[derive(Serialize)]
pub struct Group {
    pub name: String,
    pub posts: Vec<Post>,
}

#[derive(Serialize)]
pub struct Post {
    pub id: String,
    pub name: String,
    pub permalink_url: String,
    pub message: String,
    pub created_time: String,
    pub picture: String,
    pub from: From,
}

#[derive(Serialize)]
pub struct From {
    id: String,
    name: String,
}

impl Facebook
{
    pub fn new(token: &str) -> Self
    {
        Facebook {
            token: token.to_string(),
        }
    }

    pub fn group(&self, name: &str) -> Group
    {
        Group {
            name: name.to_string(),
            posts: vec![
                Post {
                    name: "Assemblée des blessé-e-s 44's".to_string(),
                    permalink_url: "https://www.facebook.com/Assemblée-des-blessé-e-s-44-1751594901723011/".to_string(),
                    message: "6 blessé.es dans les manifestations portent plainte pour violence policière dont Adrien (fracture du crâne et hémorragie cérébrale) et Philippe (rupture de la rate et hémorragie interne). Ne laissons pas passer les violences policières !".to_string(),
                    created_time: "January 11 at 5:06 AM".to_string(),
                    id: "703".to_string(),
                    picture: "703".to_string(),
                    from: From {
                        id: "10".to_string(),
                        name: "Assemblée des blessé-e-s ".to_string(),
                    },
                }
            ],
        }
    }
}
