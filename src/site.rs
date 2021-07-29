#[derive(Clone, Debug, elephantry::Entity, serde::Deserialize, serde::Serialize)]
#[elephantry(model = "Model", structure = "Structure", relation = "public.site")]
pub struct Entity {
    #[elephantry(pk)]
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

impl<'a> Model<'a> {
    pub fn find(&self, url: &str) -> elephantry::Result<Option<Entity>> {
        Ok(
            self.connection.find_where::<Self>("channel_link = $1", &[&url], None)?
                .next()
        )
    }
}
