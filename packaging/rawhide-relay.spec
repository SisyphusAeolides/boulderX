Name:           rawhide-relay
Version:        0.1.0
Release:        1%{?dist}
Summary:        GTK4 IRC client written in Rust using relm4

License:        GPL-2.0-or-later
URL:            https://github.com/SisyphusCode/rawhide-relay
Source0:        %{url}/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  pkgconfig(gtk4)
BuildRequires:  pkgconfig(openssl)

%description
Rawhide Relay is a GTK4 IRC client built in Rust with relm4.
It connects to Libera.Chat over TLS and supports NickServ auth.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release --offline

%install
install -Dm755 target/release/rawhide-relay %{buildroot}%{_bindir}/rawhide-relay

%files
%license LICENSE
%doc README.md
%{_bindir}/rawhide-relay

%changelog
* Sun Jun 21 2026 Kenny Glowner <sisyphuscode@fedoraproject.org> - 0.1.0-1
- Initial package
