/// Wraper around `url::Url` to fix serde issue
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Url(#[serde(with = "url_serde")] url::Url);

impl Url {
    pub fn parse<S: AsRef<str>>(input: S) -> std::result::Result<Url, url::ParseError> {
        url::Url::parse(input.as_ref()).map(Url)
    }
}
