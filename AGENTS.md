# logTerminator Agent Instructions

This document provides comprehensive guidance for coding agents working in the logTerminator project, a Tauri desktop application combining a Vue.js frontend with a Rust backend.

## Project Overview

logTerminator is a Tauri application that combines:
- **Frontend**: Vue 3 with Vuetify UI components
- **Backend**: Rust with Tauri framework
- **Build System**: Vite for frontend, Cargo for Rust
- **Package Manager**: pnpm
- **Database**: SQLite with Rusqlite
- **Purpose**: Log file analyzer for HTML-based test logs with bookmarking capabilities

## Build, Lint, and Test Commands

### Vue.js/Vite Frontend
```bash
# Development server
pnpm dev

# Production build
pnpm build

# Preview production build
pnpm preview
```

### Tauri Application
```bash
# Development mode (runs both frontend and backend)
pnpm tauri dev

# Production build (creates installer)
pnpm tauri build

# Bundle only (after build)
pnpm tauri bundle

# Show environment info
pnpm tauri info
```

### Rust Backend
```bash
# Run all tests
cd src-tauri && cargo test

# Run specific test
cd src-tauri && cargo test test_function_name

# Run tests with output
cd src-tauri && cargo test -- --nocapture

# Clippy checks
cd src-tauri && cargo clippy

# Auto-fix Clippy suggestions
cd src-tauri && cargo clippy --fix

# Format code
cd src-tauri && cargo fmt

# Check formatting (no changes)
cd src-tauri && cargo fmt --check

# Release mode build
cd src-tauri && cargo build --release

# Debug mode build
cd src-tauri && cargo build
```

### Running Individual Tests
- **Rust**: `cd src-tauri && cargo test test_function_name`

## Code Style Guidelines

### Vue.js Frontend

#### File Structure
```
src/
├── main.js          # Application entry point, Vuetify setup
├── App.vue          # Root component
└── assets/          # Static resources
```

#### Component Style
- Use Vue 3 Composition API with `<script setup>` syntax
- Import Vue functions from 'vue': `import { ref, computed } from 'vue'`
- Use single-file components (.vue files)
- Order: template, script, then styles

#### Imports
```javascript
// Vue imports first
import { ref, computed, onMounted } from 'vue'

// Third-party libraries
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

// Local components/resources
import SomeComponent from './components/SomeComponent.vue'
```

#### Naming Conventions
- **Components**: PascalCase (MyComponent.vue)
- **Variables/Functions**: camelCase (myVariable, myFunction)
- **Constants**: UPPER_SNAKE_CASE (MY_CONSTANT)
- **Files**: kebab-case for components (my-component.vue)

#### Templates
- HTML attributes use kebab-case: `v-model`, `@click`, `:class`
- Use shorthand for common directives: `@` for `v-on:`, `:` for `v-bind:`
- Consistent indentation (2 spaces)

#### Scripts
```javascript
<script setup>
// Reactive variables
const myData = ref('')
const isLoading = ref(false)

// Computed properties
const computedValue = computed(() => myData.value.toUpperCase())

// Functions
function handleSubmit() {
  // implementation
}

// Lifecycle hooks
onMounted(() => {
  // initialization
})
</script>
```

#### Styles
- Use scoped styles: `<style scoped>`
- CSS custom properties for theming
- Follow BEM naming when needed
- Use CSS Grid/Flexbox for responsive design

### Rust Backend

#### File Structure
```
src-tauri/
├── src/
│   ├── main.rs       # Application entry point
│   ├── lib.rs        # Tauri command library
│   ├── log_parser.rs # HTML log parsing logic
│   └── database.rs   # SQLite database operations
├── tests/
│   └── log_parser_tests.rs # Unit tests
├── Cargo.toml        # Rust dependencies
└── tauri.conf.json   # Tauri configuration
```

#### Code Style
- Follow standard Rust formatting (enforced by `cargo fmt`)
- Use `cargo clippy` for linting
- 4-space indentation (rustfmt default)
- 100-character line width (default)

#### Tauri Commands
```rust
// Command definition
#[tauri::command]
fn my_command(name: &str) -> Result<String, String> {
    // implementation
    Ok(format!("Hello, {}!", name))
}

// Handler registration in lib.rs
.invoke_handler(tauri::generate_handler![my_command])
```

#### Error Handling
- Use `Result<T, E>` for operations that can fail
- Use appropriate custom error types
- Use `?` operator for proper error propagation
- Provide meaningful error messages

#### Naming Conventions
- **Functions/Methods**: snake_case (my_function)
- **Types/Structs**: PascalCase (MyStruct)
- **Constants**: SCREAMING_SNAKE_CASE (MY_CONSTANT)
- **Modules**: snake_case (my_module)
- **Fields**: snake_case (field_name)

#### Imports
```rust
// Standard library
use std::collections::HashMap;

// External crates
use serde::{Deserialize, Serialize};
use tauri::command;

// Local modules
mod my_module;
use my_module::MyStruct;
```

#### Documentation
- Use `///` for public API documentation
- Include examples when necessary
- Document safety requirements for unsafe code

## Development Workflow

### Adding New Features

1. **Frontend Changes**:
   ```bash
   # Start development server
   pnpm dev
   # Make changes, test in browser
   ```

2. **Backend Changes**:
   ```bash
   # Test Rust code
   cd src-tauri && cargo test
   cd src-tauri && cargo clippy

   # Run full application
   pnpm tauri dev
   ```

3. **Before Commit**:
   ```bash
   # Format and lint all code
   cd src-tauri && cargo fmt
   cd src-tauri && cargo clippy

   # Build to ensure everything works
   pnpm tauri build
   ```

### Debugging

- **Frontend**: Use browser developer tools (F12)
- **Backend**: Use `println!` or `dbg!` macros, check logs in terminal
- **Full App**: Use `pnpm tauri dev` for integrated debugging

## Dependencies

### Frontend
- Vue 3: Reactive UI framework
- Vuetify 3: Material Design component library
- Vite: Fast build tool and dev server
- Tauri APIs: Communication with Rust backend
- @tauri-apps/plugin-dialog: File/directory picker
- @tauri-apps/plugin-opener: Open files/URLs

### Backend
- Tauri: Desktop application framework
- Rusqlite: SQLite database bindings (with bundled SQLite)
- Scraper: HTML parsing and CSS selectors
- Tokio: Asynchronous runtime
- Chrono: Date/time handling with serde support
- Walkdir: Directory traversal utilities
- Serde: Serialization/deserialization

## Security Considerations

- Validate all inputs between frontend and backend
- Use Tauri's security model (capabilities, permissions)
- Avoid exposing sensitive operations through Tauri commands
- Follow Rust's memory safety guarantees

## Performance Guidelines

- Minimize Tauri command calls (batch operations when possible)
- Use Vue's reactive system efficiently
- Optimize bundle size (tree-shaking enabled by default)
- Use browser dev tools and Rust profilers for analysis

## Editor Configuration

### Cursor Rules
No Cursor-specific rules found (.cursor/rules/ or .cursorrules).

### Copilot Rules
No Copilot-specific rules found (.github/copilot-instructions.md).

## File References

When referencing code locations, use the format:
- Frontend: `src/App.vue:42`
- Backend: `src-tauri/src/lib.rs:15`