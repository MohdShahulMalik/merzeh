use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct FormData {
    #[garde(dive)]
    pub identifier: UserIdentifier,
    #[garde(length(min = 8))]
    pub password: String
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub enum UserIdentifier {
    Email(
        #[garde(email)]
        String
    ),
    Mobile(
        #[garde(pattern(r"^[6-9][0-9]{9}$"))]
        String
    ),
}
