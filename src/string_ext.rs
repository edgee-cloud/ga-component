pub trait StringExt {
    fn or<'a>(&'a self, default: &'a str) -> &str;
}

impl StringExt for String {
    fn or<'a>(&'a self, default: &'a str) -> &str {
        if self.is_empty() {
            default
        } else {
            self
        }
    }
}
