# Claude Code Instructions for pctrl

## Before Committing

**IMPORTANT:** Always run these commands before committing:

```bash
# Format all Rust code
cargo fmt

# Check for warnings/errors
cargo clippy

# Verify it compiles
cargo check
```

## Code Style Rules

### Rust Formatting
- Always run `cargo fmt` after making changes to Rust files
- Do not manually format code - let rustfmt handle it
- Lines should not exceed 100 characters (rustfmt default)
- Use 4 spaces for indentation

### File Locations
- CLI commands: `apps/cli/src/main.rs`
- Command handlers: `apps/cli/src/cli.rs`
- TUI interface: `apps/cli/src/tui/` (mod.rs, app.rs, ui.rs, input.rs, types.rs)
- Core types: `crates/core/src/lib.rs`
- Database CRUD: `crates/database/src/lib.rs`

## Database Migrations

When changing the database schema:

1. **Increment version** in `crates/database/src/migrations.rs`:
   ```rust
   pub const CURRENT_SCHEMA_VERSION: i32 = 3;  // bump this
   ```

2. **Add migration function**:
   ```rust
   async fn migrate_v3(pool: &SqlitePool) -> Result<()> {
       // Add new columns, tables, etc.
       sqlx::query("ALTER TABLE ... ADD COLUMN ...")
           .execute(pool).await?;
       Ok(())
   }
   ```

3. **Register in run_migration()**:
   ```rust
   match version {
       2 => migrate_v2(pool).await,
       3 => migrate_v3(pool).await,  // add this
       _ => Ok(()),
   }
   ```

Migrations run automatically on startup. Always check if column exists before ALTER TABLE.

## Project Architecture

This project follows a **project-centric architecture** (MASTERPLAN v6):

- **Projects** are the core organizing entity
- All resources (servers, containers, databases, domains, scripts) link to projects
- Use `project_resources` table for many-to-many relationships

## Entity Types

| Entity | Description |
|--------|-------------|
| Project | Central organizing unit with status (dev/staging/live/archived) |
| Server | VPS, dedicated, or local servers |
| Domain | Domain names with DNS info |
| DatabaseCredentials | Database connection credentials |
| Container | Docker containers |
| Script | Automation scripts |
| ProjectResource | Links projects to resources |

## Common Commands

```bash
# Run TUI (default mode)
cargo run --package pctrl-cli

# Run CLI mode (status display)
cargo run --package pctrl-cli -- -m cli

# Run specific command
cargo run --package pctrl-cli -- project list

# Build all
cargo build

# Test all
cargo test
```

## CI/CD

The GitHub Actions workflow runs:
1. `cargo fmt -- --check` - Fails if code is not formatted
2. `cargo clippy` - Warns about code issues
3. `cargo test` - Runs all tests
4. `cargo build` - Builds the project

**Always run `cargo fmt` before committing to avoid CI failures.**

## After Completing a Task

**IMPORTANT:** After completing any feature or fix:

1. **Update ROADMAP.md**
   - Mark completed items with ✅
   - Update phase status if needed ([current] → [done])

2. **Update Documentation**
   - ARCHITECTURE.md - if architecture changed
   - CHANGELOG.md - add entry for the change
   - README.md - if user-facing features changed

3. **Deploy if needed**
   - Run `dep 2` (git only) or `dep 1` (git + coolify)
   - This syncs ROADMAP to landing page

Example workflow:
```bash
# 1. Complete the feature
# 2. Update ROADMAP.md (mark ✅)
# 3. Update relevant docs
# 4. cargo fmt && cargo clippy
# 5. dep 2 "feat: add deprecation warnings"
```
