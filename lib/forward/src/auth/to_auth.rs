use super::Authentication;

#[derive(PartialEq, Debug)]
pub enum ToAuthenticationError {
    NoUsername,
    NoPassword,
}

pub trait ToAuthentication {
    fn to_authentication(&self) -> Result<Authentication, ToAuthenticationError>;
}

impl ToAuthentication for (Option<String>, Option<String>) {
    fn to_authentication(&self) -> Result<Authentication, ToAuthenticationError> {
        match self {
            (Some(username), Some(password)) => Ok(Authentication::Password {
                username: username.to_string(),
                password: password.to_string(),
            }),
            (Some(_), None) => Err(ToAuthenticationError::NoPassword),
            (None, Some(_)) => Err(ToAuthenticationError::NoUsername),
            (None, None) => Ok(Authentication::None),
        }
    }
}
