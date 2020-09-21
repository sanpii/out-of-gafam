#[derive(Clone, Debug, elephantry::Entity, serde::Deserialize, serde::Serialize)]
pub struct Entity {
    pub id: Option<uuid::Uuid>,
    #[serde(default)]
    pub channel_link: String,
    #[serde(default)]
    pub channel_title: String,
    pub channel_description: Option<String>,
    pub channel_image: Option<String>,
    #[serde(default)]
    pub items: String,
    #[serde(default)]
    pub item_title: String,
    #[serde(default)]
    pub item_link: String,
    #[serde(default)]
    pub item_description: String,
    #[serde(default)]
    pub item_pubdate: String,
    pub item_guid: Option<String>,
}

pub struct Model<'a> {
    connection: &'a elephantry::Connection,
}

impl<'a> Model<'a> {
    pub fn find(&self, url: &str) -> elephantry::Result<Option<Entity>> {
        Ok(
            self.connection.find_where::<Self>("channel_link = $1", &[&url], None)?
                .nth(0)
        )
    }
}

impl<'a> elephantry::Model<'a> for Model<'a> {
    type Entity = Entity;
    type Structure = Structure;

    fn new(connection: &'a elephantry::Connection) -> Self {
        Self { connection }
    }
}

pub struct Structure;

impl elephantry::Structure for Structure
{
    fn relation() -> &'static str
    {
        "public.site"
    }

    fn primary_key() -> &'static [&'static str]
    {
        &["id"]
    }

    fn columns() -> &'static [&'static str]
    {
        &[
            "id",
            "channel_link",
            "channel_title",
            "channel_description",
            "channel_image",
            "items",
            "item_title",
            "item_link",
            "item_description",
            "item_pubdate",
            "item_guid",
        ]
    }
}
