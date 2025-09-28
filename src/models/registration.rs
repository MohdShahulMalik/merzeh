use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Validate, Deserialize)]
pub struct FormData {
    #[garde(dive)]
    pub identifier: UserIdentifier,
    #[garde(length(min = 8))]
    pub password: String
}

#[derive(Debug, Validate, Deserialize)]
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
