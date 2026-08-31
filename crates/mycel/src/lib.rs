pub mod mycel {
    pub mod common {
        pub mod v1 {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/gen/rust/mycel.common.v1.rs"
            ));
        }
    }

    pub mod client {
        pub mod v1 {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/gen/rust/mycel.client.v1.rs"
            ));
        }
    }

    pub mod admin {
        pub mod v1 {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/gen/rust/mycel.admin.v1.rs"
            ));
        }
    }
}

pub use mycel::*;
