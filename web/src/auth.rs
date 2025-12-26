use std::ffi::{OsStr, OsString};

use axum_auth::AuthBasic;
use nonstick::{AuthnFlags, ConversationAdapter, Transaction, TransactionBuilder};
use vm_types::Config;
use yave::yavecontext::YaveContext;

struct UsernamePassConvo {
    username: String,
    password: String,
}

impl ConversationAdapter for UsernamePassConvo {
    fn prompt(&self, _: impl AsRef<OsStr>) -> nonstick::Result<OsString> {
        Ok(OsString::from(&self.username))
    }

    fn masked_prompt(&self, _: impl AsRef<OsStr>) -> nonstick::Result<OsString> {
        Ok(OsString::from(&self.password))
    }

    fn error_msg(&self, _: impl AsRef<OsStr>) {
        // Normally you would want to display this to the user somehow.
        // In this case, we're just ignoring it.
    }

    fn info_msg(&self, _: impl AsRef<OsStr>) {
        // ibid.
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid creditinals")]
    InvalidCreditinals,
}

pub fn check(AuthBasic((username, password)): &AuthBasic, config: &Config) -> Result<(), Error> {
    let mut txn = TransactionBuilder::new_with_service("common_auth")
        .username(username)
        .build(UsernamePassConvo {
            password: password.as_ref().map(String::to_string).unwrap_or("".to_string()),
            username: username.to_string(),
        }.into_conversation()).map_err(|_| Error::InvalidCreditinals)?;
    txn.authenticate(AuthnFlags::empty()).map_err(|_| Error::InvalidCreditinals)?;
    txn.account_management(AuthnFlags::empty()).map_err(|_| Error::InvalidCreditinals)?;

    let user = users::get_user_by_name(username).expect("Impossible error, if pam work");
    if let Some(groups) = user.groups() {
        if !groups.iter().fold(false, |acc, x| acc || config.api.groups.contains(&x.name().to_string_lossy().to_string())) {
            return Err(Error::InvalidCreditinals);
        }
    }
    Ok(())
}