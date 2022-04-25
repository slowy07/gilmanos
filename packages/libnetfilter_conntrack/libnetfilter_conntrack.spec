Name: %{_cross_os}libnetfilter_conntrack
Version: 1.0.7
Release: 1%{?dist}
Summary: Library for netfilter conntrack
License: GPLv2+
URL: http://netfilter.org
Source0: https://netfilter.org/projects/libnetfilter_conntrack/files/libnetfilter_conntrack-%{version}.tar.bz2
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libmnl-devel
BuildRequires: %{_cross_os}libnfnetlink-devel
Requires: %{_cross_os}glibc
Requires: %{_cross_os}libmnl
Requires: %{_cross_os}libnfnetlink

%description
%{summary}.

%package devel
Summary: Files for development using the library for netfilter conntrack
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libnetfilter_conntrack-%{version} -p1

%build
%cross_configure \
  --disable-rpath \
  --enable-static

%make_build

%install
%make_install

%files
%{_cross_libdir}/*.so.*

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/libnetfilter_conntrack
%{_cross_includedir}/libnetfilter_conntrack/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
