# Tauri + Vue 3

This template should help get you started developing with Tauri + Vue 3 in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Test Log File Naming Convention

The application only parses HTML files that follow the test log naming pattern:

**Pattern:** `<TestName>_<ID_X>---<Y>.html`

- `<TestName>`: Test identifier (e.g., "TestEnableTcpdump", "TestABC")
- `<ID_X>`: Test instance ID (e.g., "ID_1")
- `<Y>`: Sequential file number (e.g., 0, 1, 2, 3...)

**Examples:**
- `TestEnableTcpdump_ID_1---0.html` → Test session: TestEnableTcpdump_ID_1
- `TestEnableTcpdump_ID_1---1.html` → Test session: TestEnableTcpdump_ID_1 (continuation)
- `TestABC_ID_1---0.html` → Test session: TestABC_ID_1

**Files that don't match this pattern are ignored**, such as:
- `MainRollup.html`
- `summary.html`
- `TestWithoutID---0.html` (missing `_ID_` pattern)

Each unique `<TestName>_<ID_X>` combination is treated as a separate test session, allowing you to analyze individual tests separately.
