## Build

This crate requires the nightly channel and the cargo feature `host_deps` enabled in order to work properly.

    cargo +nightly build -Zhost_dep

Instead of using `-Z`, you can also add the following to your `.cargo/config`:

~~~toml
[unstable]
features = ["host_dep"]
~~~
