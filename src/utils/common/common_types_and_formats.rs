use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct Xslt {
    pub xslt: Vec<u8>,
    pub compiled_xslt: Option<Vec<u8>>,
}

#[derive(Default, Debug, Clone)]
pub struct Cert {
    pub cert: Vec<u8>,
}

pub type XsltCache = HashMap<String, Xslt>;
pub type CertCache = HashMap<String, Cert>;

/*
let mut xslt_dict: XsltMap = HashMap::new();
xslt_dict.insert("invoice".into(), Xslt::default());

let mut cert_dict: CertMap = HashMap::new();
cert_dict.insert("signing".into(), Cert::default());
*/
