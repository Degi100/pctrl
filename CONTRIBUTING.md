# Contributing to pctrl

Thank you for your interest in contributing to pctrl!

## Project Overview

pctrl follows a **project-centric architecture** where projects are the core organizing entity. All resources (servers, containers, databases, domains, scripts) are linked to projects.

## Getting Started

1. Fork the repository
2. Clone your fork
3. Install dependencies:
   ```bash
   # Rust
   rustup update stable

   # Node.js (for desktop/landing/mobile)
   npm install
   ```
4. Create a new branch for your feature or bugfix
5. Make your changes
6. Test your changes
7. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70+
- Node.js 18+
- SQLite3

### Building

```bash
# Build all Rust crates
cargo build

# Build CLI only
cargo build --package pctrl-cli

# Run CLI
cargo run --package pctrl-cli

# Run TUI
cargo run --package pctrl-cli -- tui

# Run tests
cargo test
```

### Project Structure

```
pctrl/
├── apps/
│   ├── cli/src/
│   │   ├── main.rs     # CLI command definitions
│   │   ├── cli.rs      # Command handlers
│   │   └── tui.rs      # TUI with Project View
│   ├── desktop/        # Tauri + React
│   ├── landing/        # Astro website
│   └── mobile/         # Expo app
│
├── crates/
│   ├── core/           # Types: Project, Server, Domain, Database, Script
│   ├── database/       # SQLite CRUD operations
│   ├── ssh/            # SSH connection management
│   ├── docker/         # Docker API client
│   ├── coolify/        # Coolify API client
│   └── git/            # Git operations
```

## Core Entity Types

When contributing, understand the core entities:

| Entity | Location | Description |
|--------|----------|-------------|
| `Project` | `crates/core/src/lib.rs` | Central organizing entity |
| `Server` | `crates/core/src/lib.rs` | VPS, dedicated, local servers |
| `Domain` | `crates/core/src/lib.rs` | Domain names with DNS info |
| `DatabaseCredentials` | `crates/core/src/lib.rs` | Database connection info |
| `Container` | `crates/core/src/lib.rs` | Docker containers |
| `Script` | `crates/core/src/lib.rs` | Automation scripts |
| `ProjectResource` | `crates/core/src/lib.rs` | Links projects to resources |

## Adding New Features

### Adding a New CLI Command

1. Define command in `apps/cli/src/main.rs`:
   ```rust
   #[derive(Subcommand)]
   pub enum MyCommands {
       List,
       Add { name: String },
       // ...
   }
   ```

2. Add handler in `apps/cli/src/cli.rs`:
   ```rust
   pub async fn handle_my_command(cmd: MyCommands, db: &Database) -> anyhow::Result<()> {
       match cmd {
           MyCommands::List => { /* ... */ }
           MyCommands::Add { name } => { /* ... */ }
       }
   }
   ```

### Adding a New Entity Type

1. Define type in `crates/core/src/lib.rs`:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct MyEntity {
       pub id: String,
       pub name: String,
       // ...
   }
   ```

2. Add database table in `crates/database/src/lib.rs`:
   ```rust
   // In init_schema()
   sqlx::query("CREATE TABLE IF NOT EXISTS my_entities (...)")
       .execute(&self.pool)
       .await?;
   ```

3. Add CRUD methods:
   ```rust
   pub async fn save_my_entity(&self, entity: &MyEntity) -> Result<()> { /* ... */ }
   pub async fn get_my_entity(&self, id: &str) -> Result<Option<MyEntity>> { /* ... */ }
   pub async fn list_my_entities(&self) -> Result<Vec<MyEntity>> { /* ... */ }
   pub async fn remove_my_entity(&self, id: &str) -> Result<()> { /* ... */ }
   ```

### Adding TUI Panel

1. Add to `SelectedPanel` enum in `apps/cli/src/tui.rs`:
   ```rust
   enum SelectedPanel {
       // ...
       MyPanel,
   }
   ```

2. Add form fields if needed
3. Add navigation handling
4. Add content rendering
5. Add save handler

## Code Style

### Rust
- Follow standard Rust formatting (`cargo fmt`)
- Run clippy: `cargo clippy`
- Use meaningful variable and function names
- Add comments for complex logic
- Write tests for new features

### TypeScript/JavaScript
- Follow Prettier formatting
- Use TypeScript for type safety

## Commit Messages

Use conventional commits format:

```
feat(cli): add domain management commands
fix(database): correct project_resources query
docs(readme): update installation instructions
refactor(core): simplify Server type
test(database): add CRUD tests for projects
```

## Pull Request Process

1. Ensure all tests pass: `cargo test`
2. Ensure code is formatted: `cargo fmt`
3. Ensure no clippy warnings: `cargo clippy`
4. Update documentation if needed
5. Add entry to changelog if significant
6. Request review from maintainers

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test --package pctrl-core
cargo test --package pctrl-database

# With output
cargo test -- --nocapture
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_status_default() {
        let status = ProjectStatus::default();
        assert_eq!(status, ProjectStatus::Dev);
    }

    #[tokio::test]
    async fn test_save_project() {
        let db = Database::new_in_memory().await.unwrap();
        let project = Project { /* ... */ };
        db.save_project(&project).await.unwrap();
        // ...
    }
}
```

## Security

- Never commit secrets or credentials
- Use encrypted storage for sensitive data
- Follow OWASP guidelines
- Report security issues privately

## Questions?

- Open a [GitHub Discussion](https://github.com/Degi100/pctrl/discussions)
- Submit an [issue](https://github.com/Degi100/pctrl/issues)
- Check [ARCHITECTURE.md](ARCHITECTURE.md) for system design
- Check [ROADMAP.md](ROADMAP.md) for planned features
