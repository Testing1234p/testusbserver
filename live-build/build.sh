#!/bin/bash
set -e

# Hardware Agent USB - Alpine Live ISO Builder
# Eseguire in WSL2 o Linux x86_64

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$SCRIPT_DIR/build"
ISO_NAME="hardware-agent-live.iso"
ALPINE_VERSION="3.19"
ALPINE_FULL_VERSION="3.19.9"
ARCH="x86_64"

# Pacchetti da installare nella rootfs
PACKAGES="busybox openrc ipmitool smartmontools pciutils usbutils lm-sensors dmidecode beep linux-lts syslinux linux-firmware-none"

echo "========================================"
echo "  Hardware Agent USB - ISO Builder"
echo "========================================"

# Cleanup e preparazione
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"
mkdir -p "$BUILD_DIR/boot"
mkdir -p "$BUILD_DIR/rootfs"

echo "[1/7] Download Alpine minirootfs..."
cd "$BUILD_DIR"
if [ ! -f "alpine-minirootfs-${ALPINE_FULL_VERSION}-${ARCH}.tar.gz" ]; then
    wget -q "https://dl-cdn.alpinelinux.org/alpine/v${ALPINE_VERSION}/releases/${ARCH}/alpine-minirootfs-${ALPINE_FULL_VERSION}-${ARCH}.tar.gz"
fi

echo "[2/7] Estrazione rootfs..."
tar xzf "alpine-minirootfs-${ALPINE_FULL_VERSION}-${ARCH}.tar.gz" -C rootfs/

# Installa strumenti host necessari
echo "[3/7] Installazione strumenti host..."
sudo apt-get update -qq >/dev/null 2>&1 || true
sudo apt-get install -y -qq syslinux syslinux-efi syslinux-utils xorriso grub-pc-bin grub-efi-amd64-bin mtools >/dev/null 2>&1 || true

# Configura DNS per chroot
mkdir -p rootfs/etc
cp /etc/resolv.conf rootfs/etc/ || true

# Installa kernel e pacchetti nella rootfs
echo "[4/7] Installazione pacchetti nella rootfs..."

# Mount necessari per chroot
sudo mount --bind /proc rootfs/proc
sudo mount --bind /sys rootfs/sys
sudo mount --bind /dev rootfs/dev

# Aggiungi repository
cat <<EOF | sudo tee rootfs/etc/apk/repositories
https://dl-cdn.alpinelinux.org/alpine/v${ALPINE_VERSION}/main
https://dl-cdn.alpinelinux.org/alpine/v${ALPINE_VERSION}/community
EOF

# Installa pacchetti
sudo chroot rootfs /sbin/apk update
sudo chroot rootfs /sbin/apk add --no-cache $PACKAGES || true

# Configura init automatico
echo "[5/7] Configurazione avvio automatico..."

# Script init che parte automaticamente
sudo mkdir -p rootfs/etc/local.d/
sudo cp "$SCRIPT_DIR/init.sh" rootfs/etc/local.d/hardware-agent.start
sudo chmod +x rootfs/etc/local.d/hardware-agent.start
sudo chroot rootfs /sbin/rc-update add local default || true

# Crea utente root senza password per auto-login su console
sudo sed -i 's/^root:.*/root::0:0:root:\/root:\/bin\/sh/' rootfs/etc/passwd

# Configura inittab per auto-login su tty1
if [ -f rootfs/etc/inittab ]; then
    sudo sed -i 's/\/sbin\/getty/\/sbin\/getty -n -l \/bin\/sh/' rootfs/etc/inittab || true
fi

# Copia binario agente
echo "[6/7] Compilazione e installazione agente..."
cd "$PROJECT_DIR"

# Cross-compila per musl (statico)
rustup target add x86_64-unknown-linux-musl 2>/dev/null || true
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-musl

# Copia binario nella rootfs
sudo mkdir -p "$BUILD_DIR/rootfs/usr/local/bin"
sudo cp "$PROJECT_DIR/target/x86_64-unknown-linux-musl/release/hardware-agent" "$BUILD_DIR/rootfs/usr/local/bin/"
sudo chmod +x "$BUILD_DIR/rootfs/usr/local/bin/hardware-agent"

# Crea directory di mount per la chiavetta
sudo mkdir -p "$BUILD_DIR/rootfs/mnt/usb"

# Pulizia mountpoint
sudo umount "$BUILD_DIR/rootfs/proc" || true
sudo umount "$BUILD_DIR/rootfs/sys" || true
sudo umount "$BUILD_DIR/rootfs/dev" || true

# Prepara initramfs e kernel
echo "[7/7] Creazione ISO bootabile..."

# Trova kernel installato
KERNEL_FILE=$(ls "$BUILD_DIR/rootfs/boot/vmlinuz"* 2>/dev/null | head -n1)
if [ -z "$KERNEL_FILE" ]; then
    echo "[ERRORE] Kernel non trovato nella rootfs"
    exit 1
fi

sudo cp "$KERNEL_FILE" "$BUILD_DIR/boot/vmlinuz"

# Crea initramfs con mkinitfs o manualmente
INITRAMFS="$BUILD_DIR/boot/initramfs"
cd "$BUILD_DIR/rootfs"
find . | cpio -o -H newc 2>/dev/null | gzip > "$INITRAMFS.gz"
cd "$BUILD_DIR"

# Crea bootloader syslinux
mkdir -p "$BUILD_DIR/boot/syslinux"
cp /usr/lib/ISOLINUX/isolinux.bin "$BUILD_DIR/boot/syslinux/isolinux.bin"
cp /usr/lib/syslinux/modules/bios/ldlinux.c32 "$BUILD_DIR/boot/syslinux/ldlinux.c32" 2>/dev/null || true
cat <<EOF > "$BUILD_DIR/boot/syslinux/syslinux.cfg"
DEFAULT hwagent
LABEL hwagent
  MENU LABEL Hardware Agent USB
  LINUX /boot/vmlinuz
  INITRD /boot/initramfs.gz
  APPEND root=/dev/ram0 rw quiet console=tty0
TIMEOUT 10
EOF

# Crea ISO con xorriso
# Prepara directory staging per la ISO (senza rootfs)
ISO_STAGING="$BUILD_DIR/iso_staging"
mkdir -p "$ISO_STAGING"
cp -a "$BUILD_DIR/boot" "$ISO_STAGING/"

# Prepara EFI boot se possibile
mkdir -p "$ISO_STAGING/efi/boot"
cp /usr/lib/SYSLINUX.EFI/efi64/syslinux.efi "$ISO_STAGING/efi/boot/bootx64.efi" 2>/dev/null || true
cp /usr/lib/syslinux/modules/efi64/ldlinux.e64 "$ISO_STAGING/efi/boot/" 2>/dev/null || true

cd "$ISO_STAGING"

xorriso -as mkisofs \
    -o "$BUILD_DIR/$ISO_NAME" \
    -c boot/boot.cat \
    -b boot/syslinux/isolinux.bin \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    -eltorito-alt-boot \
    -e efi/boot/bootx64.efi \
    -no-emul-boot \
    -isohybrid-gpt-basdat \
    -isohybrid-mbr /usr/lib/ISOLINUX/isohdpfx.bin \
    . 2>/dev/null || \
xorriso -as mkisofs \
    -o "$BUILD_DIR/$ISO_NAME" \
    -c boot/boot.cat \
    -b boot/syslinux/isolinux.bin \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    .

echo "========================================"
echo "  BUILD COMPLETATO"
echo "  ISO: $BUILD_DIR/$ISO_NAME"
echo "========================================"
