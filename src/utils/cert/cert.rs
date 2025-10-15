#[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Cert {
    pub cert: Vec<u8>,
}
