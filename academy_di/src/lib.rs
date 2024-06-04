pub use academy_di_derive::Build;
use typemap::TypeMap;

mod macros;
mod typemap;

#[derive(Debug, Default)]
pub struct ProviderState(TypeMap);

pub trait Provider {
    fn state(&mut self) -> &mut ProviderState;
}

#[diagnostic::on_unimplemented(
    message = "The type `{Self}` cannot be built using the provider `{P}`",
    note = "Add `{Self}` to the provider `{P}` or implement `Build` for `{Self}` and make sure \
            all dependencies are satisfied"
)]
pub trait Build<P> {
    fn build(provider: &mut P) -> Self;
}

pub trait Provides<T> {
    fn provide(&mut self) -> T;
}

impl<T, P> Provides<T> for P
where
    T: Build<P> + Clone + 'static,
    P: Provider,
{
    fn provide(&mut self) -> T {
        if let Some(cached) = self.state().0.get().cloned() {
            cached
        } else {
            let object = T::build(self);
            self.state().0.insert(object.clone());
            object
        }
    }
}
