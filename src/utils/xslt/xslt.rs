#[derive(Default, Debug, Clone)]
pub struct Xslt {
    pub xslt: Vec<u8>,
    pub compiled_xslt: Option<Vec<u8>>,
}
