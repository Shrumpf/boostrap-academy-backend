#[macro_export]
macro_rules! required_action {
    ($vis:vis $ident:ident) => {
        #[must_use]
        #[derive(::std::fmt::Debug)]
        $vis struct $ident { done: bool }
        impl $ident {
            fn new() -> Self { Self { done: false } }
            $vis fn ok(mut self) { self.done = true; }
        }
        impl ::core::ops::Drop for $ident {
            fn drop(&mut self) {
                if !self.done {
                    ::core::panic!("The required action '{}' has not been performed!", ::core::stringify!($ident));
                }
            }
        }
    };

}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic = "The required action 'DoSomething' has not been performed!"]
    fn not_done() {
        let _ = DoSomething::new();
    }

    #[test]
    fn done() {
        let required = DoSomething::new();
        required.ok();
    }

    required_action!(DoSomething);
}
