---
name: auto_build_after_fix
description: 在完成修复或功能实现后，必须尝试自动编译项目
priority: 1
---

# Auto-Build Skill

## 规则（强制）

当你完成以下任一任务后：
- 修复 Bug
- 实现新功能
- 完成代码重构

并且你认为代码修改已经完成、可以验证时：

### 必须执行以下步骤：

1. 调用命令工具执行：
   `pnpm tauri build`

2. 如果编译失败：
   - 读取错误日志
   - 自动尝试修复
   - 再次执行 `pnpm tauri build`

3. 直到：
   - 编译成功
   - 或明确向用户报告失败原因
