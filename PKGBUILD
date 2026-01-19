pkgname=parton
pkgver=0.1.0
pkgrel=1
pkgdesc="Run Windows executables in a running game's Proton context"
arch=('x86_64')
url="https://github.com/eyenalxai/parton"
license=('MIT')
depends=('gcc-libs')
makedepends=('cargo')
options=('!debug' 'strip')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('a70834e2a3af6234eecddd28b66a1c06071417c79f5626708f9e0def975717ca')

prepare() {
	cd "$pkgname-$pkgver"
	export RUSTUP_TOOLCHAIN=stable
	cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
	cd "$pkgname-$pkgver"
	export RUSTUP_TOOLCHAIN=stable
	export CARGO_TARGET_DIR=target
	cargo build --frozen --release --all-features
}

package() {
	cd "$pkgname-$pkgver"
	install -Dm0755 -t "$pkgdir/usr/bin/" "target/release/$pkgname"
}
