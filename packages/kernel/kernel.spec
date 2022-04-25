%global debug_package %{nil}

Name: %{_cross_os}kernel
Version: 4.19.66
Release: 1%{?dist}
Summary: The Linux kernel
License: GPLv2 and Redistributable, no modification permitted
URL: https://www.kernel.org/
# Use latest-srpm-url.sh to get this.
Source0: https://cdn.amazonlinux.com/blobstore/d88833a42027a5779606b1da7579281dd2fde351262da02c3f6daf4ef8983b46/kernel-4.19.66-22.57.amzn2.src.rpm
Source100: config-thar
Patch0001: 0001-dm-add-support-to-directly-boot-to-a-mapped-device.patch
Patch0002: 0002-dm-init-fix-const-confusion-for-dm_allowed_targets-a.patch
Patch0003: 0003-dm-init-fix-max-devices-targets-checks.patch
Patch0004: 0004-dm-ioctl-fix-hang-in-early-create-error-condition.patch
Patch0005: 0005-dm-init-fix-incorrect-uses-of-kstrndup.patch
Patch0006: 0006-dm-init-remove-trailing-newline-from-calls-to-DMERR-.patch
Patch0007: 0007-lustrefsx-Disable-Werror-stringop-overflow.patch
BuildRequires: bc
BuildRequires: elfutils-devel
BuildRequires: gcc-%{_cross_target}
BuildRequires: hostname
BuildRequires: kmod
BuildRequires: openssl-devel

%description
%{summary}.

%package modules
Summary: Modules for the Linux kernel

%description modules
%{summary}.

%package headers
Summary: Header files for the Linux kernel for use by glibc

%description headers
%{summary}.

%prep
rpm2cpio %{SOURCE0} | cpio -iu linux-%{version}.tar config-%{_cross_arch} "*.patch"
tar -xof linux-%{version}.tar; rm linux-%{version}.tar
%setup -TDn linux-%{version}
# Patches from the Source0 SRPM
for patch in ../*.patch; do
    patch -p1 <"$patch"
done
# Patches listed in this spec (Patch0001...)
%autopatch -p1
KCONFIG_CONFIG="arch/%{_cross_karch}/configs/%{_cross_vendor}_defconfig" \
    ARCH="%{_cross_karch}" \
    scripts/kconfig/merge_config.sh ../config-%{_cross_arch} %{SOURCE100}
rm -f ../config-%{_cross_arch} ../*.patch

%global kmake \
make -s\\\
  ARCH="%{_cross_karch}"\\\
  CROSS_COMPILE="%{_cross_target}-"\\\
  INSTALL_HDR_PATH="%{buildroot}%{_cross_prefix}"\\\
  INSTALL_MOD_PATH="%{buildroot}%{_cross_prefix}"\\\
  INSTALL_MOD_STRIP=1\\\
%{nil}

%build
%kmake mrproper
%kmake %{_cross_vendor}_defconfig
%kmake %{?_smp_mflags} %{_cross_kimage}
%kmake %{?_smp_mflags} modules

%install
%kmake headers_install
%kmake modules_install

install -d %{buildroot}/boot
install -T -m 0755 arch/%{_cross_karch}/boot/%{_cross_kimage} %{buildroot}/boot/vmlinuz
install -m 0644 .config %{buildroot}/boot/config
install -m 0644 System.map %{buildroot}/boot/System.map

find %{buildroot}%{_cross_prefix} \
   \( -name .install -o -name .check -o \
      -name ..install.cmd -o -name ..check.cmd \) -delete

%files
/boot/vmlinuz
/boot/config
/boot/System.map

%files modules
%dir %{_cross_libdir}/modules
%{_cross_libdir}/modules/*

%files headers
%dir %{_cross_includedir}/asm
%dir %{_cross_includedir}/asm-generic
%dir %{_cross_includedir}/drm
%dir %{_cross_includedir}/linux
%dir %{_cross_includedir}/misc
%dir %{_cross_includedir}/mtd
%dir %{_cross_includedir}/rdma
%dir %{_cross_includedir}/scsi
%dir %{_cross_includedir}/sound
%dir %{_cross_includedir}/video
%dir %{_cross_includedir}/xen
%{_cross_includedir}/asm/*
%{_cross_includedir}/asm-generic/*
%{_cross_includedir}/drm/*
%{_cross_includedir}/linux/*
%{_cross_includedir}/misc/*
%{_cross_includedir}/mtd/*
%{_cross_includedir}/rdma/*
%{_cross_includedir}/scsi/*
%{_cross_includedir}/sound/*
%{_cross_includedir}/video/*
%{_cross_includedir}/xen/*

%changelog
