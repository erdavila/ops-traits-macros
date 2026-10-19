macro_rules! with_clone_bound {
    ($type:ty, $($return:tt)+) => {
        ops_traits_macros::__return!([$type: Clone] $($return)+)
    };
}

#[derive(Debug, Clone)]
struct S;

#[test]
fn callback_return() {
    with_clone_bound!(
        T,
        ($($bounds:tt)*) => {
            fn f<T>(x: &T) -> T
            where
                $($bounds)*
            {
                x.clone()
            }
        }
    );

    let _: S = f(&S);
}

#[test]
fn cps_return() {
    // Define the macro inside a module so that we can test macro paths.
    mod macro_module {
        macro_rules! define_function {
            ($name:ident) => {
                with_clone_bound!(
                    T,
                    macro_module::define_function!(@return $name)
                );
            };

            (@return $name:ident $($bounds:tt)*) => {
                fn $name<T>(x: &T) -> T
                where
                    $($bounds)*
                {
                    x.clone()
                }
            };
        }
        pub(crate) use define_function;
    }

    macro_module::define_function!(f);

    let _: S = f(&S);
}
