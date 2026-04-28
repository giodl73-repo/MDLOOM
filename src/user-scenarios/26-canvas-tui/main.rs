/// US-26: proof-canvas embedded in a ratatui TUI.
///
/// proof-canvas handles the fixed-width ASCII grid; ratatui handles
/// terminal I/O, event loop, and widget framing. The two integrate at
/// Canvas::render() → ratatui Paragraph.
///
/// Run:  cargo run --example canvas-tui
///
/// Cargo.toml dependencies:
///   proof-canvas = { path = "../../crates/proof-canvas" }
///   ratatui = "0.27"
///   crossterm = "0.27"

use proof_canvas::Canvas;

// ── Simulated data — in a real app these come from metrics or state. ─────────

struct AppState {
    cpu: f64,
    mem: f64,
    req_per_sec: u32,
    status: &'static str,
    scroll_offset: usize,
    log_lines: Vec<String>,
}

impl AppState {
    fn new() -> Self {
        Self {
            cpu: 42.3,
            mem: 68.1,
            req_per_sec: 1_842,
            status: "healthy",
            scroll_offset: 0,
            log_lines: vec![
                "[INFO]  2026-04-28 09:01:02  request processed  /api/v1/items  200  4ms".into(),
                "[INFO]  2026-04-28 09:01:03  request processed  /api/v1/stats  200  2ms".into(),
                "[WARN]  2026-04-28 09:01:05  slow query         /api/v1/search 200  312ms".into(),
                "[INFO]  2026-04-28 09:01:07  request processed  /api/v1/items  200  3ms".into(),
                "[INFO]  2026-04-28 09:01:08  cache hit          /api/v1/items  200  0ms".into(),
                "[ERR ]  2026-04-28 09:01:10  db timeout         /api/v1/write  500  5001ms".into(),
                "[INFO]  2026-04-28 09:01:11  retry succeeded    /api/v1/write  200  88ms".into(),
            ],
        }
    }
}

// ── Build the proof-canvas layout. ───────────────────────────────────────────
//
// Terminal is 80×24. Layout:
//   Row 0:     header bar (80×1)
//   Rows 1-5:  KPI panel (80×5)
//   Rows 6-18: log panel (80×13)
//   Row 19:    footer bar (80×1)

fn build_canvas(state: &AppState) -> Canvas {
    let mut canvas = Canvas::new(80, 20);

    // ── Header ────────────────────────────────────────────────────────────────
    let header = format!(
        "╔══ Platform Monitor ══════════════════════════════════════════════════════╗"
    );
    canvas.paste(0, 0, &[&header]);

    // ── KPI panel (rows 1-5) ─────────────────────────────────────────────────
    canvas.paste(0, 1, &["║                                                                              ║"]);
    let kpi_cpu = format!("  CPU:  {:>5.1}%", state.cpu);
    let kpi_mem = format!("  MEM:  {:>5.1}%", state.mem);
    let kpi_rps = format!("  RPS:  {:>6}", state.req_per_sec);
    let kpi_sts = format!("  Status: {}", state.status);
    let kpi_line = format!("║{:<18}{:<18}{:<18}{:<24}║", kpi_cpu, kpi_mem, kpi_rps, kpi_sts);
    canvas.paste(0, 2, &[&kpi_line]);

    // Mini bar for CPU
    let cpu_fill = ((state.cpu / 100.0) * 20.0).round() as usize;
    let cpu_bar: String = "█".repeat(cpu_fill) + &"░".repeat(20 - cpu_fill);
    let mem_fill = ((state.mem / 100.0) * 20.0).round() as usize;
    let mem_bar: String = "█".repeat(mem_fill) + &"░".repeat(20 - mem_fill);
    let bar_line = format!("║  cpu  [{}]  mem  [{}]                     ║", cpu_bar, mem_bar);
    canvas.paste(0, 3, &[&bar_line]);
    canvas.paste(0, 4, &["║                                                                              ║"]);
    canvas.paste(0, 5, &["╠══ Logs ══════════════════════════════════════════════════════════════════════╣"]);

    // ── Log panel (rows 6-18, 13 visible lines) ───────────────────────────────
    let visible = proof_canvas::scroll_clip(
        &state.log_lines.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        12,
        state.scroll_offset,
    );
    for (i, line) in visible.iter().enumerate() {
        // Truncate log line to 76 chars and wrap in border
        let truncated: String = line.chars().take(76).collect();
        let row = format!("║{:<76}║", truncated);
        canvas.paste(0, 6 + i, &[&row]);
    }
    // Pad remaining rows if fewer logs than panel height
    for i in visible.len()..12 {
        canvas.paste(0, 6 + i, &["║                                                                              ║"]);
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    canvas.paste(0, 18, &["║                                                                              ║"]);
    canvas.paste(0, 19, &["╚══ q:quit  ↑↓:scroll ══════════════════════════════════════════════════════╝"]);

    canvas
}

fn main() {
    let state = AppState::new();
    let canvas = build_canvas(&state);

    // In a real ratatui app this would be drawn via:
    //   let text = canvas.render();
    //   let paragraph = ratatui::widgets::Paragraph::new(text);
    //   frame.render_widget(paragraph, area);
    //
    // Here we just print so the example is runnable without a TTY:
    print!("{}", canvas.render());
}
