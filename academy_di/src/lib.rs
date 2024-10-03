pub use academy_di_derive::Build;
pub use typemap::TypeMap;

mod macros;
mod typemap;

pub trait Provider: Sized {
    fn get<T: 'static + Clone>(&self) -> Option<T>;
    fn insert<T: 'static>(&mut self, value: T);
}

#[diagnostic::on_unimplemented(
    message = "The type `{Self}` cannot be built using the provider `{P}`",
    note = "Add `{Self}` to the provider `{P}` or implement `Build` for `{Self}` and make sure \
            all dependencies are satisfied"
)]
pub trait Build<P: Provider>: Clone + 'static {
    fn build(provider: &mut P) -> Self;
}

pub trait Provide: Provider {
    fn provide<T: Build<Self>>(&mut self) -> T {
        T::build(self)
    }
}

impl<P: Provider> Provide for P {}
