Name: %{_cross_os}readline
Version: 8.0
Release: 1%{?dist}
Summary: A library for editing typed command lines
License: GPLv3+
URL: https://tiswww.case.edu/php/chet/readline/rltop.html
Source0: https://ftp.gnu.org/gnu/readline/readline-%{version}.tar.gz
Patch1: readline-8.0-shlib.patch
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}ncurses-devel
Requires: %{_cross_os}glibc
Requires: %{_cross_os}ncurses

%description
%{summary}.

%package devel
Summary: Files for development using a library for editing typed command lines
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n readline-%{version} -p1

%build
%cross_configure --with-curses --disable-install-examples
%make_build

%install
%make_install

%files
%{_cross_libdir}/*.so.*
%exclude %{_cross_infodir}
%exclude %{_cross_mandir}
%exclude %{_cross_datadir}/doc/readline/*

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/readline
%{_cross_includedir}/readline/*.h
%{_cross_pkgconfigdir}/*.pc

%changelog
