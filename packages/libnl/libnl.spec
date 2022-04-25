%global rpmver 3.4.0
%global srcver 3_4_0

Name: %{_cross_os}libnl
Version: %{rpmver}
Release: 1%{?dist}
Summary: Convenience library for netlink
License: LGPLv2+
URL: https://github.com/thom311/libnl
Source0: https://github.com/thom311/libnl/archive/libnl%{srcver}.tar.gz
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}glibc

%description
%{summary}.

%package devel
Summary: Files for development using the convenience library for netlink
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libnl-libnl%{srcver} -p1

%build
autoreconf -fi
%cross_configure \
  --enable-static \
  --disable-cli \

sed -i 's|^hardcode_libdir_flag_spec=.*|hardcode_libdir_flag_spec=""|g' libtool
sed -i 's|^runpath_var=LD_RUN_PATH|runpath_var=DIE_RPATH_DIE|g' libtool

%make_build

%install
%make_install

%files
%{_cross_libdir}/*.so.*
%exclude %{_cross_mandir}
%exclude %{_cross_sysconfdir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/libnl3
%{_cross_includedir}/libnl3
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
