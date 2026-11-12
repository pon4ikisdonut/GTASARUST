# Platform & Driver Support

## Supported Targets (x86_64 ONLY)
- `x86_64-pc-windows-msvc` (Windows 10 / 11 x64): DX12 & Vulkan via `wgpu`, WASAPI audio via `kira`/`cpal`, XInput/RawInput/DirectInput.
- `x86_64-unknown-linux-gnu` (Ubuntu, Debian, Fedora, Arch Linux x86_64): Vulkan via `wgpu`, ALSA/PulseAudio/PipeWire via `kira`/`cpal`, X11 & Wayland via `winit`, evdev via `gilrs`.

## Required Linux Build Packages
- Debian/Ubuntu: `pkg-config libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxkbcommon-dev libwayland-dev libasound2-dev libudev-dev libdbus-1-dev`
- Fedora: `pkg-config libX11-devel libXcursor-devel libXrandr-devel libXi-devel libxkbcommon-devel wayland-devel alsa-lib-devel systemd-devel dbus-devel`
- Arch Linux: `pkgconf libx11 libxcursor libxrandr libxi libxkbcommon wayland alsa-lib systemd-libs dbus`
