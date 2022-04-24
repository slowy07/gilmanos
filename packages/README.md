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