use crate::app_id::RefAppId;
use std::{
    fmt::{self, Display},
    num::NonZeroUsize,
};

pub struct Host<'a> {
    app_id: &'a RefAppId,
    dsn: bool,
    backup_number: Option<NonZeroUsize>,
}

impl<'a> Host<'a> {
    pub fn new(app_id: &'a RefAppId) -> Self {
        Self {
            app_id,
            dsn: false,
            backup_number: None,
        }
    }

    pub fn with_dsn(app_id: &'a RefAppId, dsn: bool) -> Self {
        Self {
            app_id,
            dsn,
            backup_number: None,
        }
    }

    /// Note: backup_number of `0` is the same thing as `None`
    pub fn with_backup(app_id: &'a RefAppId, backup_number: Option<usize>) -> Self {
        Self {
            app_id,
            dsn: false,
            backup_number: backup_number.and_then(NonZeroUsize::new),
        }
    }
}

impl<'a> Display for Host<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.app_id.as_ref())?;
        if self.dsn {
            f.write_str("-dsn")?;
        }

        if let Some(backup_number) = self.backup_number {
            write!(f, "-{}", backup_number)?;
        }

        f.write_str(".algolia.net")
    }
}
