# Maintainer: gaster4432
pkgname=shell-shooter
pkgver=0.2.1
pkgrel=1
pkgdesc='A fast 2D arcade space shooter written in Rust'
arch=('x86_64' 'aarch64')
url='https://github.com/gaster4432/shell-shooter'
license=('MIT')
depends=('gcc-libs' 'glibc' 'libglvnd' 'libx11' 'libxi' 'libxcursor' 'alsa-lib')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('0234d8538bfa4396ff2c47981450063baf4e846c91a9d870a9de2c8f2cbed0d0')

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
