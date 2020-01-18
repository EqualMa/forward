#[derive(PartialEq, Debug, Clone)]
pub enum Authentication {
    Password { username: String, password: String },
    None,
}

impl Authentication {
    pub fn id(&self) -> u8 {
        match self {
            Authentication::Password { .. } => 2,
            Authentication::None => 0,
        }
    }

    pub fn is_no_auth(&self) -> bool {
        Authentication::None == *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_eq() {
        assert_eq!(
            Authentication::Password {
                username: "usr".to_string(),
                password: "pwd".to_string(),
            },
            Authentication::Password {
                username: "usr".to_string(),
                password: "pwd".to_string(),
            },
        );

        assert_eq!(Authentication::None, Authentication::None);
    }

    #[test]
    fn auth_id() {
        assert_eq!(
            Authentication::Password {
                username: "usr".to_string(),
                password: "pwd".to_string(),
            }
            .id(),
            2,
        );
        assert_eq!(Authentication::None.id(), 0);
    }

    #[test]
    fn no_auth() {
        assert!(Authentication::None.is_no_auth());
        assert!(!Authentication::Password {
            username: "usr".to_string(),
            password: "pwd".to_string(),
        }
        .is_no_auth());
    }
}
