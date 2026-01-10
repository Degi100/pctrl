//! CLI Styling - FinanzApp Quality
//!
//! Farben, Banner und formatierte Ausgaben für professionellen CLI-Look.

// ANSI Color Codes
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";

// Colors (allow dead_code - will be used in CLI handlers)
#[allow(dead_code)]
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
#[allow(dead_code)]
pub const YELLOW: &str = "\x1b[33m";
#[allow(dead_code)]
pub const BLUE: &str = "\x1b[34m";
#[allow(dead_code)]
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";
pub const GRAY: &str = "\x1b[90m";

/// Print the pctrl ASCII banner
pub fn print_banner(version: &str) {
    println!(
        r#"
{g}  ┌───────────────────────────────────────────────────────────┐
  │                                                           │
  │   {w}██████╗  {c}██████╗{w}████████╗{c}██████╗ {w}██╗{g}                     │
  │   {w}██╔══██╗{c}██╔════╝{w}╚══██╔══╝{c}██╔══██╗{w}██║{g}                     │
  │   {w}██████╔╝{c}██║        {w}██║   {c}██████╔╝{w}██║{g}                     │
  │   {w}██╔═══╝ {c}██║        {w}██║   {c}██╔══██╗{w}██║{g}                     │
  │   {w}██║     {c}╚██████╗   {w}██║   {c}██║  ██║{w}███████╗{g}                │
  │   {w}╚═╝      {c}╚═════╝   {w}╚═╝   {c}╚═╝  ╚═╝{w}╚══════╝{g}                │
  │                                                           │
  │   {c}Mission Control for Self-Hosters{g}          {d}v{v}{g}   │
  │                                                           │
  └───────────────────────────────────────────────────────────┘{r}
"#,
        g = GREEN,
        w = WHITE,
        c = CYAN,
        d = DIM,
        r = RESET,
        v = version
    );
}

/// Print a section header (like FinanzApp's build phases)
#[allow(dead_code)]
pub fn section(title: &str) {
    println!();
    println!(
        "{}┌─────────────────────────────────────────┐{}",
        BLUE, RESET
    );
    println!(
        "{}│{} {}{}{:<39}{} {}│{}",
        BLUE, RESET, WHITE, BOLD, title, RESET, BLUE, RESET
    );
    println!(
        "{}└─────────────────────────────────────────┘{}",
        BLUE, RESET
    );
}

/// Print a step indicator [1/4]
#[allow(dead_code)]
pub fn step(current: u32, total: u32, text: &str) {
    println!();
    println!(
        "{}[{}/{}]{} {}{}{}",
        YELLOW, current, total, RESET, BOLD, text, RESET
    );
}

/// Success message with checkmark
#[allow(dead_code)]
pub fn success(msg: &str) {
    println!("{}✓{} {}", GREEN, RESET, msg);
}

/// Error message with X
#[allow(dead_code)]
pub fn error(msg: &str) {
    println!("{}✗{} {}", RED, RESET, msg);
}

/// Warning message
#[allow(dead_code)]
pub fn warn(msg: &str) {
    println!("{}⚠{}  {}", YELLOW, RESET, msg);
}

/// Info message
#[allow(dead_code)]
pub fn info(msg: &str) {
    println!("{}ℹ{}  {}", CYAN, RESET, msg);
}

/// Print a key-value pair (for status display)
#[allow(dead_code)]
pub fn kv(key: &str, value: &str) {
    println!("  {}{}:{} {}", GRAY, key, RESET, value);
}

/// Print a key-value pair with count (highlighted if > 0)
pub fn kv_count(key: &str, count: usize) {
    let (icon, value) = if count > 0 {
        ("●", format!("{}{}{}", GREEN, count, RESET))
    } else {
        ("○", format!("{}0{}", DIM, RESET))
    };
    println!("  {} {}: {}", icon, key, value);
}

/// Print a horizontal divider
pub fn divider() {
    println!("{}─────────────────────────────────────────{}", GRAY, RESET);
}

/// Print a boxed summary (like FinanzApp's success banner)
#[allow(dead_code)]
pub fn success_box(title: &str, lines: &[&str]) {
    println!();
    println!(
        "{}  ┌───────────────────────────────────────────┐{}",
        GREEN, RESET
    );
    println!(
        "{}  │{}  {}{}{:<39}{}  {}│{}",
        GREEN, RESET, WHITE, BOLD, title, RESET, GREEN, RESET
    );
    println!(
        "{}  ├───────────────────────────────────────────┤{}",
        GREEN, RESET
    );
    for line in lines {
        println!("{}  │{}  {:<41}  {}│{}", GREEN, RESET, line, GREEN, RESET);
    }
    println!(
        "{}  └───────────────────────────────────────────┘{}",
        GREEN, RESET
    );
}

/// Format a path for display (truncate if too long)
pub fn format_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - max_len + 3..])
    }
}
