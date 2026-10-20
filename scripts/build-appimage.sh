#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "$0")/.." && pwd)
VERSION=$(sed -n 's/^version = "\([^"]*\)".*/\1/p' "$ROOT/Cargo.toml" | head -n 1)
ARCH=${ARCH:-x86_64}
TARGET=${TARGET:-x86_64-unknown-linux-gnu}
APPDIR="$ROOT/dist/UniKey_RS-$VERSION.AppDir"
OUTPUT="$ROOT/dist/UniKey_RS-$VERSION-$ARCH.AppImage"

if command -v rustup >/dev/null 2>&1 && rustup target list --installed | grep -qx 'x86_64-unknown-linux-musl'; then
    TARGET=x86_64-unknown-linux-musl
elif [ "${UNIKEY_RS_MUSL:-0}" = 1 ]; then
    TARGET=x86_64-unknown-linux-musl
fi

echo "Building UniKey RS $VERSION for $TARGET"
rm -rf "$APPDIR"
mkdir -p \
    "$APPDIR/usr/bin" \
    "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/scalable/apps" \
    "$APPDIR/usr/share/metainfo" \
    "$ROOT/dist/tools"

(
    cd "$ROOT"
    cargo build --release --locked --target "$TARGET"
)
cp "$ROOT/target/$TARGET/release/unikey-rs" "$APPDIR/usr/bin/"
cp "$ROOT/packaging/AppRun" "$APPDIR/AppRun"
cp "$ROOT/packaging/org.unikeyrs.UniKeyRS.desktop" "$APPDIR/"
cp "$ROOT/packaging/org.unikeyrs.UniKeyRS.desktop" "$APPDIR/usr/share/applications/"
cp "$ROOT/packaging/unikey-rs.svg" "$APPDIR/unikey-rs.svg"
cp "$ROOT/packaging/unikey-rs.svg" "$APPDIR/usr/share/icons/hicolor/scalable/apps/"
cp "$ROOT/packaging/org.unikeyrs.UniKeyRS.appdata.xml" "$APPDIR/usr/share/metainfo/"
chmod +x "$APPDIR/AppRun" "$APPDIR/usr/bin/unikey-rs"

APPIMAGETOOL=${APPIMAGETOOL:-$(command -v appimagetool || true)}
if [ -z "$APPIMAGETOOL" ]; then
    APPIMAGETOOL="$ROOT/dist/tools/appimagetool-$ARCH.AppImage"
    if [ ! -x "$APPIMAGETOOL" ]; then
        echo "Downloading continuous appimagetool into dist/tools"
        env -u PYTHONHOME -u PYTHONPATH -u LD_LIBRARY_PATH \
            curl -fL --retry 3 \
            "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-$ARCH.AppImage" \
            -o "$APPIMAGETOOL"
        chmod +x "$APPIMAGETOOL"
    fi
fi

ARCH=$ARCH env -u PYTHONHOME -u PYTHONPATH -u LD_LIBRARY_PATH "$APPIMAGETOOL" --appimage-extract-and-run "$APPDIR" "$OUTPUT"
sha256sum "$OUTPUT" > "$OUTPUT.sha256"
echo "Created $OUTPUT"
