//! Thread safe wrappers around ssh2 types

use ssh2::{Session, Channel};

pub struct ThreadSafeChannel<'a> {
    pub channel: Channel<'a>
}

unsafe impl <'a> Send for ThreadSafeChannel<'a> {}
unsafe impl <'a> Sync for ThreadSafeChannel<'a> {}
impl <'a> Clone for ThreadSafeChannel<'a> {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}



pub struct ThreadSafeSession {
    pub session: Session
}

unsafe impl Send for ThreadSafeSession {}
unsafe impl Sync for ThreadSafeSession {}
impl Clone for ThreadSafeSession {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}
