use base64::{Engine as _, engine::general_purpose};
use rand::{Rng, thread_rng};

pub fn generate_token() -> String {
    let mut token_bytes = [0u8; 32];
    thread_rng().fill(&mut token_bytes);

    general_purpose::URL_SAFE_NO_PAD.encode(token_bytes)
}
