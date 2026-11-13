use crate::{Error, Result};

pub(crate) fn nonblank<'a>(value: &'a str, field: &'static str) -> Result<&'a str> {
    let value = value.trim();
    if value.is_empty() {
        Err(Error::Validation { field })
    } else {
        Ok(value)
    }
}
