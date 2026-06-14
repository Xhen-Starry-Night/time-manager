# 全面审查修复清单

## 1. MemoryQuality::from_str 添加英文别名

**文件:** `src/data/models.rs:29-37`

修复 `from_str` 只接受中文的问题，增加英文别名。

## 2. CLI 读取 Config 配置

**文件:** `src/bin/tmd.rs:84,122,177`

在 CLI 启动时加载 `data::config::Config`，使用 `config.default_preset` 替代硬编码 `"default"`。

## 3. 移除废弃的 models::Config

**文件:** `src/data/models.rs:332-345`

`models::Config` 已被 `config::Config` 替代，删除未使用的结构体。

## 4. 修复 GUI RefreshPredictions 覆盖 preset_used

**文件:** `src/app/mod.rs:808-813`

`RefreshPredictions` 不应修改卡片的 `preset_used`，应保留原有值。

## 5. CLI timer 错误类型修正

**文件:** `src/bin/tmd.rs:307-328`

将 `String` 错误改为通过 `DataError::Io` 或直接 `eprintln!` + 退出。

## 6. schedule-create 添加 --priority 参数

**文件:** `src/bin/tmd.rs:413`
**文件:** `src/cli/mod.rs` (命令定义)

为 `ScheduleCreate` 子命令添加可选的 `--priority` 参数。
