# UI改进与Element Plus迁移设计

**Date:** 2026-01-23
**Status:** Design Approved
**Author:** Collaborative Design Session

## 概述

对logTerminator应用进行全面UI改进，包括：
1. 添加Session删除功能
2. 调整书签栏高度与log table一致
3. 移除log table的复选框
4. 改进分页组件，高亮当前页
5. 迁移到Element Plus组件库
6. 改进所有圆形按钮为可见的文字+图标按钮

## 动机

当前界面存在以下问题：
- Session移到导航栏后无法删除
- 书签栏高度固定，与log table不协调
- Log table的复选框没有实际用途
- 分页组件当前页不够突出
- 圆形图标按钮太隐蔽，用户难以发现

## 设计方案

### 1. 整体架构

**组件库迁移：**
- 从Vuetify 3迁移到Element Plus
- 保持现有的数据流和状态管理（Vue 3 Composition API）
- 使用Element Plus的对应组件替换Vuetify组件

**组件映射：**
| Vuetify 3 | Element Plus |
|-----------|--------------|
| `v-data-table` | `el-table` |
| `v-select` | `el-select` |
| `v-pagination` | `el-pagination` |
| `v-card` | `el-card` |
| `v-btn` (icon) | `el-button` (text/带图标) |
| `v-app-bar` | 自定义header容器 |
| `v-container` + `v-row` + `v-col` | `el-container` + 布局 |

**保留的部分：**
- Tauri后端调用（`invoke`）
- SQLite数据库逻辑
- localStorage持久化
- 响应式状态管理

### 2. Session删除功能

**在下拉菜单中添加删除按钮：**

```vue
<el-select v-model="currentSession">
  <template #default="{ item }">
    <div class="session-item">
      <div class="session-info">
        <el-icon><Folder /></el-icon>
        <span>{{ item.name }}</span>
        <span class="count">{{ item.total_entries }} 条记录</span>
      </div>
      <el-button
        type="danger"
        :icon="Delete"
        size="small"
        text
        @click.stop="deleteSession(item.id)">
      </el-button>
    </div>
  </template>
</el-select>
```

**功能：**
- 点击删除按钮时阻止事件冒泡（`@click.stop`）
- 删除前弹出确认对话框
- 删除成功后：
  - 如果删除的是当前session，清空log列表
  - 重新加载session列表
  - 显示成功提示

**数据流：**
1. 用户点击删除按钮
2. 调用后端`delete_session`命令
3. 前端更新sessions列表
4. 如果是当前session，重置currentSession

### 3. 书签栏高度与按钮改进

**书签栏高度：**
```vue
<!-- 使用固定高度，与log table容器一致 -->
<el-card class="bookmarks-panel">
  <div class="bookmarks-header">...</div>
  <div class="bookmarks-list" style="height: calc(100vh - 280px); overflow-y: auto;">
    <!-- 书签列表 -->
  </div>
</el-card>
```

**圆形按钮改进 - 使用文字按钮+图标：**

| 原Vuetify | Element Plus改进 |
|-----------|------------------|
| `<v-btn icon color="grey"><v-icon>mdi-pencil</v-icon></v-btn>` | `<el-button type="primary" :icon="Edit" link>编辑</el-button>` |
| `<v-btn icon color="grey"><v-icon>mdi-delete</v-icon></v-btn>` | `<el-button type="danger" :icon="Delete" link>删除</el-button>` |
| `<v-btn icon><v-icon>mdi-chevron-down</v-icon></v-btn>` | `<el-button :icon="ArrowDown" link>展开</el-button>` |
| 星形图标按钮 | `<el-button :icon="Star" link>收藏</el-button>` |

**设计原则：**
- 优先使用`link`类型的文字按钮
- 图标+文字组合，更清晰可见
- 颜色明确：编辑=蓝色，删除=红色
- 悬停时有背景色变化

### 4. Log表格改进

**移除复选框：**
```vue
<el-table
  :data="logEntries"
  :height="tableHeight"
  stripe>

  <!-- 时间戳列 -->
  <el-table-column prop="timestamp" label="时间戳" width="180" />

  <!-- 级别列 -->
  <el-table-column prop="level" label="级别" width="100">
    <template #default="{ row }">
      <el-tag :type="getLevelType(row.level)">{{ row.level }}</el-tag>
    </template>
  </el-table-column>

  <!-- 消息列 -->
  <el-table-column prop="message" label="消息" show-overflow-tooltip />

  <!-- 操作列 - 星标按钮 -->
  <el-table-column label="操作" width="120" align="center">
    <template #default="{ row }">
      <el-button
        :icon="isBookmarked(row.id) ? StarFilled : Star"
        :type="isBookmarked(row.id) ? 'warning' : 'default'"
        link
        @click="toggleBookmark(row)">
        {{ isBookmarked(row.id) ? '已收藏' : '收藏' }}
      </el-button>
    </template>
  </el-table-column>
</el-table>
```

**改进点：**
1. ✅ 移除复选框列
2. 星标按钮改为文字+图标，更明显
3. 使用`el-tag`显示日志级别，带颜色区分
4. 消息列支持tooltip显示完整内容

### 5. 分页组件改进

**使用Element Plus分页，数字方块样式：**

```vue
<el-pagination
  v-model:current-page="currentPage"
  :page-size="itemsPerPage"
  :total="totalEntries"
  :page-sizes="[10, 20, 50, 100]"
  layout="total, sizes, prev, pager, next, jumper"
  background
  @current-change="handlePageChange"
  @size-change="handleSizeChange">
</el-pagination>
```

**样式特点：**
- `background` 属性：数字按钮有背景色
- 当前页：蓝色背景（Element Plus主题色）
- 其他页：灰色背景
- 悬停时颜色加深

**布局说明：**
- `total` - 显示总条数
- `sizes` - 每页显示条数选择器
- `prev` - 上一页按钮
- `pager` - 页码数字（高亮当前页）
- `next` - 下一页按钮
- `jumper` - 跳转到指定页输入框

### 6. 实施计划

**迁移方式：逐步替换，保持功能完整**

**阶段1：准备工作**
1. 安装Element Plus依赖
2. 保留Vuetify，两套库共存
3. 创建Element Plus基础布局框架

**阶段2：核心组件迁移**
1. 替换导航栏（app-bar → el-header）
2. 替换Session选择器（v-select → el-select + 删除按钮）
3. 替换Log表格（v-data-table → el-table）
4. 替换分页组件（v-pagination → el-pagination）
5. 替换书签面板（v-card → el-card + 高度调整）

**阶段3：功能添加与按钮改进**
1. Session删除功能实现
2. 书签栏高度调整为calc(100vh - 280px)
3. 所有圆形按钮改为文字+图标样式

**阶段4：清理**
1. 移除Vuetify依赖
2. 清理未使用的样式
3. 全面测试所有功能

**风险控制：**
- 每个阶段完成后测试所有功能
- 保持git提交小步快跑
- 如有问题可随时回滚

**预计改动文件：**
- `src/App.vue` - 主要迁移文件
- `src/main.js` - 引入Element Plus
- `package.json` - 依赖更新

### 7. 测试清单

- [ ] Session选择器正常工作
- [ ] Session删除功能正常
- [ ] 删除当前session时正确处理
- [ ] 书签栏高度与log table一致
- [ ] Log table无复选框
- [ ] 星标按钮可见且可用
- [ ] 分页当前页高亮正确
- [ ] 所有文字按钮可见且功能正常
- [ ] 侧边栏调整宽度功能正常
- [ ] 响应式布局正常

### 8. Element Plus图标映射

| Vuetify (MDI) | Element Plus |
|---------------|--------------|
| `mdi-pencil` | `Edit` |
| `mdi-delete` | `Delete` |
| `mdi-chevron-down` | `ArrowDown` |
| `mdi-star` | `Star` |
| `mdi-folder` | `Folder` |
| `mdi-web` | `Link` |
