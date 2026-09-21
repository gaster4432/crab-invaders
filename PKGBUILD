# Maintainer: gaster4432
pkgname=crab-invaders
pkgver=0.2.1
pkgrel=1
pkgdesc='A fast 2D arcade space shooter written in Rust'
arch=('x86_64' 'aarch64')
url='https://github.com/gaster4432/crab-invaders'
license=('MIT')
depends=('gcc-libs' 'glibc' 'libglvnd' 'libx11' 'libxi' 'libxcursor' 'alsa-lib')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('d90aa0dc33f66916bee5cbfb29f487ac1ddaf5734bee09f9424a6acf40e24366')

prepare() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_INCREMENTAL=0
  cargo build --release --locked
}

check() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo test --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 "$pkgname.desktop" "$pkgdir/usr/share/applications/$pkgname.desktop"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
