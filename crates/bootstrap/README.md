<!-- cargo-sync-rdme title [[ -->
# scuffle-bootstrap
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/scuffle-bootstrap.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/scuffle-bootstrap.svg?logo=docs.rs&style=flat-square)](https://docs.rs/scuffle-bootstrap)
[![crates.io](https://img.shields.io/crates/v/scuffle-bootstrap.svg?logo=rust&style=flat-square)](https://crates.io/crates/scuffle-bootstrap)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
A utility crate for creating binaries.

Refer to [`Global`](https://docs.rs/scuffle-bootstrap/0.1.6/scuffle_bootstrap/global/trait.Global.html), [`Service`](https://docs.rs/scuffle-bootstrap/0.1.6/scuffle_bootstrap/service/trait.Service.html), and [`main`](https://docs.rs/scuffle-bootstrap/0.1.6/scuffle_bootstrap/macro.main.html) for more information.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags

### Usage

````rust,no_run
use std::sync::Arc;

/// Our global state
struct Global;

// Required by the signal service
impl scuffle_signal::SignalConfig for Global {}

impl scuffle_bootstrap::global::GlobalWithoutConfig for Global {
    async fn init() -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self))
    }
}

/// Our own custom service
struct MySvc;

impl scuffle_bootstrap::service::Service<Global> for MySvc {
    async fn run(self, global: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        println!("running");

        // Do some work here

        // Wait for the context to be cacelled by the signal service
        ctx.done().await;
        Ok(())
    }
}

// This generates the main function which runs all the services
scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        MySvc,
    }
}
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
