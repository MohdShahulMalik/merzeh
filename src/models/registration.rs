use garde::Validate;

#[derive(Debug, Validate)]
pub struct FormData {
    #[garde(dive)]
    identifier: UserIdentifier,
    #[garde(length(min = 8))]
    password: String
}

#[derive(Debug, Validate)]
pub enum UserIdentifier {
    Email(
        #[garde(
            email,
            message = "Please enter a valid email address."
        )]
        String
    ),
    Mobile(
        #[garde(
            pattern(r"^[6-9]\d{9}$"),
            message = "Please enter a valid 10 digit Indian mobile number"
        )]
        String
    ),
}
