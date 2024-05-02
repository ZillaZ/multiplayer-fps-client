use deku::prelude::*;

#[derive(DekuRead, DekuWrite)]
pub struct NewSessionRequest {
    #[deku(update = "self.id.len()")]
    id_count: usize,
    #[deku(count = "id_count")]
    pub id: Vec<u8>,
    #[deku(update = "self.password.len()")]
    count: usize,
    #[deku(count = "count")]
    pub password: Vec<u8>,
    pub player_limit: u8,
}

impl NewSessionRequest {
    pub fn new(id: &str, password: &str) -> Self {
        Self {
            id_count: id.len(),
            id: id.as_bytes().to_vec(),
            count: password.len(),
            password: password.as_bytes().to_vec(),
            player_limit: 8
        }
    }
}

#[derive(DekuRead, DekuWrite)]
pub struct JoinSessionRequest {
    #[deku(update = "self.id.len()")]
    id_count: usize,
    #[deku(count = "id_count")]
    pub id: Vec<u8>,
    #[deku(update = "self.password.len()")]
    count: usize,
    #[deku(count = "count")]
    pub password: Vec<u8>,
}

impl JoinSessionRequest {
    pub fn new(id: &str, password: &str) -> Self {
        Self {
            id_count: id.len(),
            id: id.as_bytes().to_vec(),
            count: password.len(),
            password: password.as_bytes().to_vec()
        }
    }
}

#[derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum ServerRequest {
    #[deku(id = "0x1")]
    NewSession(NewSessionRequest),
    #[deku(id = "0x2")]
    JoinSession(JoinSessionRequest),
}

#[derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum JoinResponse {
    #[deku(id = "0x1")]
    Ok,
    #[deku(id = "0x2")]
    WrongPassword,
}