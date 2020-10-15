# Drogue IoT BSEC interface

[![Matrix](https://img.shields.io/matrix/drogue-iot:matrix.org)](https://matrix.to/#/#drogue-iot:matrix.org)

A crate to interface with the [Bosch Sensortec Environmental Cluster (BSEC)](https://www.bosch-sensortec.com/software-tools/software/bsec/)
library:

> BSEC library provides higher-level signal processing and fusion for the BME680. The library receives compensated sensor values from the sensor API. In order to fully provide the requested sensor outputs, Bosch Sensortec environmental cluster processes the BME680 signals and combines them with the additional phone sensors.

## License

The BSEC library itself is **not** open source, it comes with a proprietary license and no source code.

So it is not possible to include the library, the source code, the header files, or any derived work from that in this
repository. Therefore, it is also not possible to provide a crate, from e.g. `crates.io`.

However, the BSEC library is distributed by Bosch on GitHub at [BoschSensortec/BSEC-Arduino-library](https://github.com/BoschSensortec/BSEC-Arduino-library).
You still need to accept the license terms in order to use it, but this makes it easier to integrate the library
into your own project.

This repository references the "BSEC-Arduino-library" repository as a Git submodule. If you check out the repository
recursively, you will also check out the "BSEC-Arduino-library":

    git clone --recursive https://github.com/drogue-iot/drogue-bsec

You can also define a dependency in your `Cargo.toml` using Git:

~~~toml
[dependencies]
drogue-bsec = { version = "0.1", git = "https://github.com/drogue-iot/drogue-bsec", branch="main" }
~~~

See:

* [Product page](https://www.bosch-sensortec.com/software-tools/software/bsec/)
* [License terms](https://www.bosch-sensortec.com/media/boschsensortec/downloads/bsec/2017-07-17_clickthrough_license_terms_environmentalib_sw_clean.pdf) (as of the time of writing)
* [BSEC-Arduino-library](https://github.com/BoschSensortec/BSEC-Arduino-library) in GitHub from Bosch Sensortec
* [Git Submodules](https://www.git-scm.com/book/en/v2/Git-Tools-Submodules)

## Example



## Build

This crate requires the nightly channel, and the cargo feature `host_deps` enabled, in order to work properly.

Add the following to the `Cargo.toml` of you binary project:

~~~toml
[unstable]
features = ["host_dep"]
~~~

Then, run cargo with `+nightly`:

    cargo +nightly build

This is required due to issue [cargo#5730](https://github.com/rust-lang/cargo/issues/5730). 

In a nutshell: Most likely you are going to build this crate for a target platform that does not match your host
platform, and your target is most likely `no_std`. However, this crate requires the use of `bindgen`, at build time
(due to the reasons explained above). The dependencies of `bindgen` however pollute the dependency tree,
and make it depend on `std`.

Using `host_dep` resolves this issue.

See:

* https://github.com/rust-lang/cargo/issues/7915
* https://github.com/rust-lang/cargo/pull/7820
