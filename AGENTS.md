# Agent Instructions for logTerminator

This document provides comprehensive guidelines for agentic coding agents working on the logTerminator project, a Tauri desktop application with Vue.js frontend and Rust backend.

## Project Overview

logTerminator is a Tauri application that combines:
- **Frontend**: Vue 3 with Vuetify UI components
- **Backend**: Rust with Tauri framework
- **Build System**: Vite for frontend, Cargo for Rust
- **Package Manager**: pnpm

## Build, Lint, and Test Commands

### Frontend (Vue.js/Vite)
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

# Production build (creates installers)
pnpm tauri build

# Bundle only (after building)
pnpm tauri bundle

# Show environment information
pnpm tauri info
```

### Rust Backend
```bash
# Run all tests
cd src-tauri && cargo test

# Run specific test
cd src-tauri && cargo test test_name

# Run tests with output
cd src-tauri && cargo test -- --nocapture

# Lint with Clippy
cd src-tauri && cargo clippy

# Auto-fix Clippy suggestions
cd src-tauri && cargo clippy --fix

# Format code
cd src-tauri && cargo fmt

# Check formatting without changes
cd src-tauri && cargo fmt --check

# Build in release mode
cd src-tauri && cargo build --release

# Build debug mode
cd src-tauri && cargo build
```

### Running Single Tests
- **Rust**: `cd src-tauri && cargo test test_function_name`
- **No JavaScript/Vue tests configured** - add test frameworks if needed

## Code Style Guidelines

### Vue.js Frontend

#### File Structure
```
src/
├── main.js          # App entry point with Vuetify setup
├── App.vue          # Root component
└── assets/          # Static assets
```

#### Component Style
- Use Vue 3 Composition API with `<script setup>` syntax
- Import Vue functions from 'vue': `import { ref, computed } from 'vue'`
- Use single-file components (.vue files)
- Template, script, and style sections in that order

#### Imports
```javascript
// Vue imports first
import { ref, computed, onMounted } from 'vue'

// Third-party libraries
import { invoke } from '@tauri-apps/api/core'

// Local components/assets
import SomeComponent from './components/SomeComponent.vue'
```

#### Naming Conventions
- **Components**: PascalCase (MyComponent.vue)
- **Variables/Functions**: camelCase (myVariable, myFunction)
- **Constants**: UPPER_SNAKE_CASE (MY_CONSTANT)
- **Files**: kebab-case for components (my-component.vue)

#### Templates
- Use kebab-case for HTML attributes: `v-model`, `@click`, `:class`
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
  // Implementation
}

// Lifecycle hooks
onMounted(() => {
  // Initialization
})
</script>
```

#### Styles
- Use scoped styles: `<style scoped>`
- CSS custom properties for theming
- Follow BEM-like naming when needed
- Responsive design with CSS Grid/Flexbox

### Rust Backend

#### File Structure
```
src-tauri/
├── src/
│   ├── main.rs       # Application entry point
│   ├── lib.rs        # Library with Tauri commands
│   └── [other modules]
├── Cargo.toml        # Rust dependencies
└── tauri.conf.json   # Tauri configuration
```

#### Code Style
- Follow standard Rust formatting (enforced by `cargo fmt`)
- Use `cargo clippy` for linting
- 4 spaces indentation (default rustfmt)
- 100 character line width (default)

#### Tauri Commands
```rust
// Command definition
#[tauri::command]
fn my_command(name: &str) -> Result<String, String> {
    // Implementation
    Ok(format!("Hello, {}!", name))
}

// Handler registration in lib.rs
.invoke_handler(tauri::generate_handler![my_command])
```

#### Error Handling
- Use `Result<T, E>` for fallible operations
- Custom error types when appropriate
- Proper error propagation with `?` operator
- Meaningful error messages

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
- Include examples when helpful
- Document safety requirements for unsafe code

## Development Workflow

### Adding New Features

1. **Frontend Changes**:
   ```bash
   # Start dev server
   pnpm dev
   # Make changes, test in browser
   ```

2. **Backend Changes**:
   ```bash
   # Test Rust code
   cd src-tauri && cargo test
   cd src-tauri && cargo clippy

   # Run full app
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

- **Frontend**: Use browser dev tools (F12)
- **Backend**: Use `println!` or `dbg!` macros, check logs in terminal
- **Full App**: Use `pnpm tauri dev` for integrated debugging

## Dependencies

### Frontend
- Vue 3: Reactive UI framework
- Vuetify: Material Design component library
- Vite: Fast build tool and dev server
- Tauri API: Communication with Rust backend

### Backend
- Tauri: Desktop application framework
- Serde: Serialization/deserialization
- Tauri Plugin Opener: File opening capabilities

## Security Considerations

- Validate all inputs from frontend to backend
- Use Tauri's security model (capabilities, permissions)
- Avoid exposing sensitive operations via Tauri commands
- Follow Rust's memory safety guarantees

## Performance Guidelines

- Minimize Tauri command calls (batch operations when possible)
- Use Vue's reactivity system efficiently
- Optimize bundle size (tree shaking enabled by default)
- Profile with browser dev tools and Rust profilers

## File References

When referencing code locations, use the format:
- Frontend: `src/App.vue:42`
- Backend: `src-tauri/src/lib.rs:15`