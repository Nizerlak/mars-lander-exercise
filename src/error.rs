struct Error {
    reason: String,
    inner: Option<Box<Error>>,
}

impl Error {
    fn new(reason: String) -> Self {
        Self {
            reason,
            inner: None,
        }
    }

    fn with_inner(self, reason: String) -> Self {
        Self {
            reason,
            inner: Some(Box::new(self)),
        }
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        let Self { reason, inner } = self;
        if let Some(inner) = inner {
            format!("{reason} -> {}", inner.to_string())
        } else {
            reason.clone()
        }
    }
}
