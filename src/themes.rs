macro_rules! themes {
    ($($name:literal),*) => {
        pub fn load(theme: &str) -> Option<&'static str> {
            match theme {
                $(
                    $name => Some(include_str!(concat!("../themes/", $name, ".css")).as_ref()),
                )*
                _ => None,
            }
        }

        pub fn list() -> Vec<&'static str> {
            vec![ $($name),* ]
        }
    }
}

themes!("light", "dark");
