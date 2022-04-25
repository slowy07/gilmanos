Name: %{_cross_os}libxcrypt
Version: 4.4.3
Release: 1%{?dist}
Summary: Extended crypt library for descrypt, md5crypt, bcrypt, and others
License: LGPLv2+ and BSD and Public Domain
URL: https://github.com/besser82/libxcrypt
Source0: https://github.com/besser82/libxcrypt/archive/v%{version}/libxcrypt-%{version}.tar.gz
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}glibc

%description
%{summary}.

%package devel
Summary: Files for development using the extended crypt library for descrypt, md5crypt, bcrypt, and others
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libxcrypt-%{version} -p1
./bootstrap

%build
%cross_configure \
  --disable-failure-tokens \
  --disable-valgrind \
  --disable-silent-rules \
  --enable-hashes=all \
  --enable-obsolete-api=no \
  --enable-obsolete-api-enosys=no \
  --enable-shared \
  --enable-static \
  --with-pkgconfigdir=%{_cross_pkgconfigdir} \

%make_build

%install
%make_install

%files
%{_cross_libdir}/*.so.*
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
