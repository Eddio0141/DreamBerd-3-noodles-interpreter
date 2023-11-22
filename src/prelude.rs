use std::ops::Deref;

pub struct Wrapper<T>(pub T);

impl<T> Deref for Wrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
