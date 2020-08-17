use std::{
    borrow::{Borrow, BorrowMut},
    fmt::{self, Display},
    ops::{Deref, DerefMut},
};

#[derive(Clone, Debug)]
// TODO: make an invariant that this _must_ be valid visible-ascii
pub struct AppId(String);

impl AppId {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<RefAppId> for AppId {
    fn as_ref(&self) -> &RefAppId {
        self.0.as_str().into()
    }
}

impl AsMut<RefAppId> for AppId {
    fn as_mut(&mut self) -> &mut RefAppId {
        self.0.as_mut_str().into()
    }
}

impl Deref for AppId {
    type Target = RefAppId;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for AppId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl Borrow<RefAppId> for AppId {
    fn borrow(&self) -> &RefAppId {
        self.as_ref()
    }
}

impl BorrowMut<RefAppId> for AppId {
    fn borrow_mut(&mut self) -> &mut RefAppId {
        self.as_mut()
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct RefAppId(str);

impl RefAppId {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl<'a> From<&'a str> for &'a RefAppId {
    fn from(s: &'a str) -> Self {
        // SAFE: RefAppId is `repr(transparent)` of a str
        unsafe { &*(s as *const str as *const RefAppId) }
    }
}

impl<'a> From<&'a mut str> for &'a mut RefAppId {
    fn from(s: &'a mut str) -> Self {
        // SAFE: RefAppId is `repr(transparent)` of a str
        unsafe { &mut *(s as *mut str as *mut RefAppId) }
    }
}

impl AsRef<str> for RefAppId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsMut<str> for RefAppId {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl ToOwned for RefAppId {
    type Owned = AppId;
    fn to_owned(&self) -> Self::Owned {
        AppId(self.0.to_owned())
    }
}

impl Display for RefAppId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}
