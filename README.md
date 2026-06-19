# Explorer

基于 [Iced](https://iced.rs) 的 Windows 资源管理器风格文件浏览程序。

## 项目结构

```
Explorer/
├── Cargo.toml          # 工作区 + GUI 二进制
├── locales/            # Fluent 翻译资源
│   ├── en.ftl
│   └── zh-Hans.ftl
├── explorer-core/      # 非 UI 核心逻辑库
│   └── src/
│       ├── entry.rs        # 文件条目模型
│       ├── fs.rs           # 文件系统操作
│       ├── i18n/           # LanguageBundle、message id、ICU 格式化
│       ├── model.rs        # 浏览状态与命令处理
│       └── navigation.rs   # 前进/后退历史
└── src/                # Iced 界面
    ├── main.rs         # 程序入口
    ├── message/        # 顶层 Message（嵌套各模块子消息）
    │   ├── mod.rs
    │   ├── explorer.rs # 导航、地址栏
    │   ├── file_list.rs # 文件列表
    │   ├── tree.rs     # 目录树
    │   ├── theme.rs    # 主题
    │   └── input.rs    # 键盘
    ├── theme.rs        # 主题选择与解析
    ├── tasks.rs        # 异步任务转换
    ├── style.rs        # 共享样式
    ├── widget/         # 可复用控件
    │   ├── directory_tree.rs
    │   └── file_list.rs
    ├── app/            # 应用状态与 update 逻辑
    │   ├── mod.rs
    │   └── update.rs   # 按模块分发 update
    └── view/           # 视图组件
        ├── mod.rs      # 布局编排
        ├── toolbar.rs
        └── status_bar.rs
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
- explorer-core（文件系统与导航逻辑，无 UI 依赖）
- Fluent、ICU4X（翻译与 locale 感知格式化）
