pub mod mycel {
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("mycel.common.v1");
        }
    }

    pub mod client {
        pub mod v1 {
            tonic::include_proto!("mycel.client.v1");
        }
    }

    pub mod admin {
        pub mod v1 {
            tonic::include_proto!("mycel.admin.v1");
        }
    }
}

pub use mycel::*;
