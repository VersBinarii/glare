pub trait AtCommand<'a> {
    fn cmd(&self) -> &'static str;
    fn data(&self) -> Option<&str>;
    fn with_data(&mut self, _data: Option<&'a str>) {}
}

macro_rules! impl_command {
    ($cmd_name:ident, $cmd:expr) => {
        pub struct $cmd_name<'a> {
            data: Option<&'a str>,
        }

        impl<'a> core::default::Default for $cmd_name<'a> {
            fn default() -> Self {
                Self { data: None }
            }
        }

        impl<'a> AtCommand<'a> for $cmd_name<'a> {
            fn cmd(&self) -> &'static str {
                concat!("AT+", $cmd)
            }

            fn data(&self) -> Option<&str> {
                self.data
            }
            fn with_data(&mut self, data: Option<&'a str>) {
                self.data = data;
            }
        }
    };
    ($cmd_name:ident, $cmd:expr, $data:expr) => {
        pub struct $cmd_name;

        impl core::default::Default for $cmd_name {
            fn default() -> Self {
                Self
            }
        }

        impl<'a> AtCommand<'a> for $cmd_name {
            fn cmd(&self) -> &'static str {
                concat!("AT+", $cmd)
            }

            fn data(&self) -> Option<&str> {
                Some($data)
            }
        }
    };
}

impl_command!(CwModeQuery, "CWMODE?");
impl_command!(CwModeSet, "CWMODE=", "1");
