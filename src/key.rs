use std::fmt;

use crate::request::VirtualKeyRestrictions;

#[derive(Clone)]
// TODO: make an invariant that this _must_ be valid visible-ascii
pub struct ApiKey(pub String);

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ApiKey").field(&"***").finish()
    }
}

impl ApiKey {
    /// Generate a virtual key from a "real" key.
    ///
    /// A virtual key is a key that is created without making a request to the algolia server, they are sub keys of other keys, the admin key cannot be used as a parent key.
    ///
    /// # Examples
    /// ```
    /// let parent_key = algolia::ApiKey("Example Key".to_owned());
    /// let virtual_key = parent_key.generate_virtual_key(&Default::default());
    /// assert_eq!(virtual_key.0, "MDBlNTFhZmY1Y2IxM2Q4NDk3OWM2ZGQ0YTEzODAyODE4NDE4ZThjM2U4Mjg1YjNiZGY1YjIxNGM2N2JmODE0Y3VzZXJUb2tlbj0=");
    /// ```
    pub fn generate_virtual_key(&self, restrictions: &VirtualKeyRestrictions) -> ApiKey {
        use hmac::{Hmac, Mac, NewMac};

        let mut restrictions = serde_urlencoded::to_string(&restrictions)
            .expect("We control `restrictions`' format, it shouldn't error");

        // HACK: algolia doesn't understand empty `restrictions`
        if restrictions.is_empty() {
            restrictions = "userToken=".to_string();
        }

        let restrictions = restrictions;

        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(self.0.as_bytes())
            .expect("HMAC can take key of any size");

        mac.update(&restrictions.as_bytes());

        // note: we aren't doing any equality checks, so the warning doesn't apply.
        let key = mac.finalize().into_bytes();

        // we need to first convert the raw bytes into a hex string
        let mut key = hex::encode(key);

        // then merge it with the restrictions from earlier
        key.push_str(&restrictions);

        // then base 64 encode it
        let key = base64::encode(key);

        ApiKey(key)
    }
}
