# Maintainer: Tomás Barros <tomasbarros1@gmail.com>

pkgname=ddc_brightness
pkgver=0.1.1
pkgrel=1
pkgdesc="Lightweight desktop application for controlling monitor brightness via DDC/CI"
arch=('x86_64')
url="https://github.com/tomas-barros1/ddc_brightness"
license=('MIT')
depends=('ddcutil' 'gtk4' 'libadwaita')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/tomas-barros1/ddc_brightness/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release --frozen
}

check() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo test --frozen
}

package() {
    cd "$srcdir/$pkgname-$pkgver"
    install -Dm755 target/release/ddc_brightness "$pkgdir/usr/bin/ddc_brightness"
    install -Dm644 ddc_brightness.desktop "$pkgdir/usr/share/applications/ddc_brightness.desktop"
}
