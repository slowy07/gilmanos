# Gilmanos packages

this document describe ho packaages are built for gilmanos

like any linux distribution, gilmanos builds on a foundation of open source software, from linux kernel through the GNU C library and more. unlike most distributions, it is not designed to be self-hosting. it to package and maintain only the softare that eventually ship, and not the software needed to build that software.

## development

for the say called ``libwoof``, the C library that provides the reference implementation for the woof framework.

### structure

this litring show the directory structure of sample package
```
packages/libwoof/
├── Cargo.toml
├── build.rs
├── pkg.rs
├── libwoof.spec
```
each pacakge has a ``Cargo.toml`` file that lit its depedencies, along with metadata such as external filename and the expected hashes.

it also include a ``build.rs`` [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) which tells cargo to invoke our ``buildsys`` tool. the RPM packages are built as a side effect of cargo running the script.

**Cargo.toml**

Our Sample package has the following manifest.

```
[package]
name = "libwoof"
version = "0.1.0"
edition = "2018"
publish = false
build = "build.rs"

[lib]
path = "pkg.rs"

[[package.metadata.build-package.external-files]]
url = "http://downloads.sourceforge.net/libwoof/libwoof-1.0.0.tar.xz"
sha512 = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"

# RPM BuildRequires
[build-dependencies]
glibc = { path = "../glibc" }
libseccomp = { path = "../libseccomp" }

# RPM Requires
[dependencies]
# None
```
be sure to include ``publsh = false`` for all packages, as these are not standard create and should never appear on [crates](https://crates.io/).

**Metadata**

The [packages.metadata](https://doc.rust-lang.org/cargo/reference/manifest.html#the-metadata-table-optional) table is ignored by Cargo and interpreted by our ``buildsys`` tool.


it contains an ``external-files`` list which provides upstream URLs and expected hashes. These files are, by default, only fetched from our upstream source mirror, using the URL template.

**Depedencies**

we use the dependencies and build-dependencies section of ``Cargo.toml`` to ensure additional package are built.


**build.rs**

we use the same build script for all packages.

```rs
use std::process::{exit, command};

fn main() -> Result<(), std::io::Error> {
    let ret = Command::new("buildsys").arg("build-package").statu()?;
    if !ret.success() {
        exit(1);
      }
      Ok(())
  }
```

