# UniKey RS

**Bí danh beta: `UniKey RS`. Dòng phiên bản: `0.x`.**

UniKey RS là bộ gõ văn bản viết bằng Rust, có CLI và giao diện Wayland native dùng chung một engine composition. Bản beta mặc định bật tiếng Việt.

## Tính năng chính

- Tiếng Anh/US, tiếng Việt Telex, VNI và VIQR.
- GUI Wayland native dùng `xdg-shell` + `wl_shm`, không phụ thuộc GTK/Qt.
- Input Method v2, tự quay về v1 khi compositor cũ hơn.
- Đọc keymap XKB từ compositor; bảng QWERTY chỉ là phương án dự phòng.
- Cấu hình dùng chung cho CLI, GUI và input method.

## Chạy thử

```sh
cargo run -- layouts
cargo run -- compose --layout telex 'Xin chaof'
cargo run -- doctor
cargo run -- gui
cargo run -- input-method
```

Đổi sang giao diện English:

```sh
unikey-rs config-set language en
```

Đổi lại tiếng Việt:

```sh
unikey-rs config-set language vi
```

## AppImage trên Fedora Atomic

```sh
chmod +x UniKey_RS-0.1.2-x86_64.AppImage
./UniKey_RS-0.1.2-x86_64.AppImage doctor
./UniKey_RS-0.1.2-x86_64.AppImage gui
```

Không cần cài system package và không ghi vào `/usr`. Xem `docs/FEDORA_ATOMIC.md` để tích hợp desktop ở cấp người dùng.

## Trạng thái

Đây là bản `0.x` đang phát triển. Chỉ phát hành `1.0` khi sản phẩm cực kỳ trưởng thành, gần như không còn gì cần cập nhật và hỗ trợ tốt Linux, macOS, Windows.
