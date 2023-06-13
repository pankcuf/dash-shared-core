pub mod boxer;
pub mod get_status_response;
pub mod unboxer;
pub mod opaque_runtime;

pub trait FromFFI {
    type Item: ToFFI;
    /// # Safety
    unsafe fn decode(&self) -> Self::Item;
}

pub trait ToFFI {
    type Item: FromFFI;
    fn encode(&self) -> Self::Item;
}
