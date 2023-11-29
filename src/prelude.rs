use std::{fmt::Debug, ops::Deref};

#[derive(Debug)]
pub struct Wrapper<T: Debug>(pub T);

impl<T: Debug> Deref for Wrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
