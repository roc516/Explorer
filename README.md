# Explorer

基于 [Iced](https://iced.rs) 的 Windows 资源管理器风格文件浏览程序。

## 项目结构

```
Explorer/
├── Cargo.toml              # workspace
├── crates/
│   ├── explorer/           # Iced GUI 二进制
│   ├── explorer-app/       # 应用状态、i18n、展示逻辑
│   │   └── locales/        # Fluent 翻译资源
│   ├── explorer-core/      # 文件系统抽象与路径类型
│   └── backends/
│       ├── explorer-fs-folder/ # 本地目录后端
│       └── explorer-fs-zip/    # ZIP 后端
└── scripts/icon/           # 桌面图标与 .desktop 安装脚本
```

## 功能

- 工具栏：后退、前进、上级、刷新
- 地址栏：输入路径并跳转
- 左侧边栏：可展开/折叠的目录树（驱动器与子文件夹）
- 文件列表：名称、修改日期、类型、大小
- 单击选中，双击打开文件夹或系统默认程序打开文件
- 快捷键：Enter 打开、Backspace 上级、F5 刷新、←/→ 历史导航
- 多语言：Fluent 资源文件 + ICU 格式化；跟随系统 / 简体中文 / English

## 运行

```bash
cargo run --release
```

## 依赖

- Rust 2021
- Iced 0.14（仅 GUI 层）
- explorer-core（文件系统抽象，无 UI 依赖）
- explorer-app（应用状态与 i18n）
- Fluent、ICU4X（翻译与 locale 感知格式化）
