# pet-rs — 桌面宠物（原型）

这是一个最小原型，使用 Rust + `winit` + `pixels` 创建无边框透明窗口，渲染逐帧动画（示例为程序生成的占位帧），并实现基于像素 alpha 的点击穿透检测与拖拽窗口。

构建与运行：

```powershell
cd c:\Users\bujih\Desktop\code\github\ellenp2p\pet-rs
cargo build --release
target\release\pet-rs.exe
```

注意：当前版本使用像素 alpha 判定做交互穿透；若需系统级鼠标穿透（透明区域完全透传），需要为每个平台添加原生实现（Windows/macOS/Linux）。

自定义精灵表：
- 将你的精灵表放到 `assets/spritesheet.png`，支持水平或垂直的帧条（每帧大小一致）。程序会自动分割并载入帧；否则使用内置占位动画。

