/// Built-in symbol library — core and extended tiers.
pub struct SymbolEntry {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub chars: &'static str,
    pub width: usize,
    pub tier: SymbolTier,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SymbolTier {
    Core,
    Extended,
    Domain,
}

pub static BUILT_IN_SYMBOLS: &[SymbolEntry] = &[
    // Core — Status/KPI
    SymbolEntry {
        name: "checkmark",
        aliases: &["check", "tick", "ok"],
        chars: "✓",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "x",
        aliases: &["cross", "no", "fail"],
        chars: "✗",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "warning",
        aliases: &["warn", "caution"],
        chars: "⚠",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "info",
        aliases: &["information"],
        chars: "ℹ",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "dot",
        aliases: &["bullet", "filled"],
        chars: "●",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "dot-open",
        aliases: &["circle"],
        chars: "○",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "diamond",
        aliases: &["rhombus"],
        chars: "◆",
        width: 1,
        tier: SymbolTier::Core,
    },
    // Core — Stars
    SymbolEntry {
        name: "star",
        aliases: &["star-filled"],
        chars: "★",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "star-open",
        aliases: &["star-empty"],
        chars: "☆",
        width: 1,
        tier: SymbolTier::Core,
    },
    // Core — Arrows
    SymbolEntry {
        name: "arrow-right",
        aliases: &["right", "next"],
        chars: "→",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "arrow-left",
        aliases: &["left", "back"],
        chars: "←",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "arrow-up",
        aliases: &["up", "increase"],
        chars: "↑",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "arrow-down",
        aliases: &["down", "decrease"],
        chars: "↓",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "triangle-up",
        aliases: &["up-triangle"],
        chars: "▲",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "triangle-down",
        aliases: &["down-triangle"],
        chars: "▼",
        width: 1,
        tier: SymbolTier::Core,
    },
    // Core — Rules
    SymbolEntry {
        name: "rule-thin",
        aliases: &["divider", "rule"],
        chars: "─",
        width: 1,
        tier: SymbolTier::Core,
    },
    SymbolEntry {
        name: "rule-double",
        aliases: &["double-rule"],
        chars: "═",
        width: 1,
        tier: SymbolTier::Core,
    },
    // Extended — more arrows/stars/math
    SymbolEntry {
        name: "arrow-right-double",
        aliases: &["implies", "double-right"],
        chars: "⇒",
        width: 1,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "arrow-both",
        aliases: &["bidirectional"],
        chars: "↔",
        width: 1,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "star-4",
        aliases: &["four-star"],
        chars: "✦",
        width: 1,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "sparkle",
        aliases: &["glitter"],
        chars: "✧",
        width: 1,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "check-box",
        aliases: &["checkbox-on"],
        chars: "☑",
        width: 1,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "box",
        aliases: &["checkbox-off"],
        chars: "☐",
        width: 1,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "lightning",
        aliases: &["bolt", "power"],
        chars: "⚡",
        width: 1,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "approx",
        aliases: &["approximately"],
        chars: "≈",
        width: 1,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "not-equal",
        aliases: &["neq", "ne"],
        chars: "≠",
        width: 1,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "delta",
        aliases: &["change"],
        chars: "Δ",
        width: 1,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "infinity",
        aliases: &["inf"],
        chars: "∞",
        width: 1,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "rule-dotted",
        aliases: &["dots"],
        chars: "·",
        width: 1,
        tier: SymbolTier::Extended,
    },
    // Extended — emoji (width 2)
    SymbolEntry {
        name: "trophy",
        aliases: &["cup", "championship"],
        chars: "🏆",
        width: 2,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "target",
        aliases: &["goal", "kpi"],
        chars: "🎯",
        width: 2,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "chart-up",
        aliases: &["growth"],
        chars: "📈",
        width: 2,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "chart-down",
        aliases: &["decline"],
        chars: "📉",
        width: 2,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "fire",
        aliases: &["hot", "trending"],
        chars: "🔥",
        width: 2,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "circle-green",
        aliases: &["go", "healthy"],
        chars: "🟢",
        width: 2,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "circle-yellow",
        aliases: &["warn-circle"],
        chars: "🟡",
        width: 2,
        tier: SymbolTier::Extended,
    },
    SymbolEntry {
        name: "circle-red",
        aliases: &["stop", "critical"],
        chars: "🔴",
        width: 2,
        tier: SymbolTier::Extended,
    },
    // Domain — Sports (IceLines)
    SymbolEntry {
        name: "puck",
        aliases: &["hockey"],
        chars: "🏒",
        width: 2,
        tier: SymbolTier::Domain,
    },
    SymbolEntry {
        name: "goal",
        aliases: &["net"],
        chars: "🥅",
        width: 2,
        tier: SymbolTier::Domain,
    },
    SymbolEntry {
        name: "medal-gold",
        aliases: &["first"],
        chars: "🥇",
        width: 2,
        tier: SymbolTier::Domain,
    },
    SymbolEntry {
        name: "medal-silver",
        aliases: &["second"],
        chars: "🥈",
        width: 2,
        tier: SymbolTier::Domain,
    },
    SymbolEntry {
        name: "medal-bronze",
        aliases: &["third"],
        chars: "🥉",
        width: 2,
        tier: SymbolTier::Domain,
    },
];

/// Look up a symbol by canonical name or alias (case-insensitive).
pub fn lookup(name: &str) -> Option<&'static SymbolEntry> {
    let lower = name.to_lowercase();
    BUILT_IN_SYMBOLS
        .iter()
        .find(|e| e.name == lower.as_str())
        .or_else(|| {
            BUILT_IN_SYMBOLS
                .iter()
                .find(|e| e.aliases.contains(&lower.as_str()))
        })
}
