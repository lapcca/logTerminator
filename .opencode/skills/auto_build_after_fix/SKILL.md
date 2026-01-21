---
name: auto_build_after_fix
description: 当修复问题或者实现所有功能后自动编译
priority: 1
---

# Auto-Build Skill

## Context
当用户请求修复 Bug、实现新功能或进行代码重构时，且 AI 认为当前的代码逻辑修改已经完成。

## Protocol (强制规约)
1. **动作序列**：
   - 执行命令：`pnpm tauri build` 。
2. **验证结果**：
   - 只有当编译输出的退出码为 `0` 时，才允许向用户发送 "Task Complete"。

## Examples
- User: "修复这个拼写错误。"
- Assistant: (修改代码) -> (执行 pnpm tauri build) -> "修复完成并已成功重新编译。"