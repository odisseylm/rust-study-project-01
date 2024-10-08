
/// It contains type aliases visible outside to easy use them
/// without (the same crate) version conflicts.
///
pub mod rustainers {

    // root types
    pub use ::rustainers::{Container, ContainerId, ContainerStatus};
    pub use ::rustainers::{ExposedPort};
    pub use ::rustainers::{IdError, ImageId, ImageName, ImageNameError, ImageReference};
    pub use ::rustainers::{HealthCheck, LogMatcher, Network};
    pub use ::rustainers::{Port, PortError, SCAN_PORT_DEFAULT_TIMEOUT};
    pub use ::rustainers::{VersionError, Volume, VolumeError, VolumeName};
    pub use ::rustainers::{WaitStrategy};

    pub mod compose {
        pub use ::rustainers::compose::{ComposeContainers, ComposeError, ComposeRunOption};
        pub use ::rustainers::compose::{RunnableComposeContainers, RunnableComposeContainersBuilder};
        pub use ::rustainers::compose::{TempDirError, TemporaryDirectory, TemporaryFile};
        pub use ::rustainers::compose::{ToRunnableComposeContainers};

        // Currently there is no interesting for me.
        // pub mod images {
        //     pub use ::rustainers::compose::images::{KafkaSchemaRegistry, Redpanda};
        // }
    }

    pub mod images {
        pub use ::rustainers::images::{Alpine, GenericImage, Minio, Mongo, Postgres, Redis};
    }

    pub mod runner {
        pub use ::rustainers::runner::{ContainerError, Docker, Runner, RunnerError, RunOption};
        // Currently there is no interesting for me.
        // pub use ::rustainers::runner::{Nerdctl, Podman};
    }

    pub mod tools {
        pub use ::rustainers::tools::CopyError;
    }
}

pub mod indexmap {
    pub use ::indexmap::{IndexMap, IndexSet, Equivalent, TryReserveError};
    // macros
    pub use ::indexmap::{indexmap, indexset};
}