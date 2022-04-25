%global goproject github.com/containernetworking
%global gorepo plugins
%global goimport %{goproject}/%{gorepo}

%global gover 0.8.0
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}cni-%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: Plugins for container networking
License: ASL 2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
BuildRequires: git
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}golang
Requires: %{_cross_os}glibc
Requires: %{_cross_os}iptables

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
export BUILDTAGS="rpm_crashtraceback"
for d in $(find plugins -mindepth 2 -maxdepth 2 -type d ! -name windows) ; do
  go build -buildmode pie -tags="${BUILDTAGS}" -o "bin/${d##*/}" %{goimport}/${d}
done

%install
install -d %{buildroot}%{_cross_factorydir}/opt/cni/bin
install -p -m 0755 bin/* %{buildroot}%{_cross_factorydir}/opt/cni/bin

%files
%dir %{_cross_factorydir}/opt/cni/bin
%{_cross_factorydir}/opt/cni/bin/loopback
%{_cross_factorydir}/opt/cni/bin/bandwidth
%{_cross_factorydir}/opt/cni/bin/bridge
%{_cross_factorydir}/opt/cni/bin/dhcp
%{_cross_factorydir}/opt/cni/bin/firewall
%{_cross_factorydir}/opt/cni/bin/flannel
%{_cross_factorydir}/opt/cni/bin/host-device
%{_cross_factorydir}/opt/cni/bin/host-local
%{_cross_factorydir}/opt/cni/bin/ipvlan
%{_cross_factorydir}/opt/cni/bin/macvlan
%{_cross_factorydir}/opt/cni/bin/portmap
%{_cross_factorydir}/opt/cni/bin/ptp
%{_cross_factorydir}/opt/cni/bin/sbr
%{_cross_factorydir}/opt/cni/bin/static
%{_cross_factorydir}/opt/cni/bin/tuning
%{_cross_factorydir}/opt/cni/bin/vlan

%changelog
