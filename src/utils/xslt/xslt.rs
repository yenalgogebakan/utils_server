#[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Xslt {
    pub xslt: Vec<u8>,
    pub compiled_xslt: Option<Vec<u8>>,
}
