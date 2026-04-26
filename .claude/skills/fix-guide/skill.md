---
name: fix-guide
description: Read a proof rich JSON report and generate a fix plan. For each diagnostic, uses the context block to decide fix direction and confidence, then writes a plan.json for `proof fix --plan`.
user_invocable: true
---

# Fix Guide — AI-Assisted Fix Plan Generator

Reads `rich.json` (from `proof check --format rich`) and generates `plan.json`
for `proof fix --plan`. This is Stage 2 of the three-stage pipeline.

## Usage

```bash
# Stage 1: generate the rich report
proof check --format rich --config proof.toml . -o rich.json

# Stage 2 (this skill): review report and generate plan
# Run: /fix-guide rich.json plan.json

# Stage 3: preview then apply
proof fix --plan plan.json --dry-run
proof fix --plan plan.json --min-confidence high
```

## Input

Args: `<rich.json> <plan.json>` — paths to the input report and output plan.

## Steps

### 1. Read the rich report

```bash
cat <rich-json-path>
```

Parse the JSON array of diagnostics. Note the total count and which files are affected.

### 2. Group by file

For each unique file in the report, collect all its diagnostics. Process one file at a time.

### 3. For each diagnostic, decide the fix

Read the `rich.context` block. Use this decision matrix:

**ascii_box_width — row too wide or too narrow:**
- Look at `actual_cols` vs `expected_cols`
- Look at `lines[failing_line]` — count characters
- If one extra char at end: remove it → `confidence: high`
- If one char short: find the shortest cell and add a space → `confidence: high`
- If multiple chars off: flag as `confidence: low` (structural redesign needed)

**ascii_box_width — bottom border wrong width:**
- Compare `lines[box_bottom]` to `border_line` (top border)
- If bottom has extra trailing `+` or `-`: remove → `confidence: high`
- If bottom is clearly shorter: add fill to match top width → `confidence: high`

**ascii_box_col — column separator at wrong position:**
- `expected_cols` tells you WHERE the `|` should be
- `actual_cols` tells you where it IS
- Difference = how many chars to add/remove before the misaligned `|`
- If off by 1-2 chars: find which cell content to pad → `confidence: high`
- If off by 3+ chars or multiple separators wrong: `confidence: medium`

**ascii_cell_padding — missing cell padding:**
- `lines[failing_line]` shows the content
- Add a space after `|` (left padding) or before `|` (right padding)
- Always `confidence: high` — unambiguous

**md_missing_section — required heading absent:**
- Cannot auto-fix structural requirements → `confidence: low`
- Include in plan with description only; author must add section

### 4. Build the edit

For each high/medium confidence fix:
```json
{
  "line": <1-based line number>,
  "old_string": "<exact current content of the line>",
  "new_string": "<the corrected line>"
}
```

**Rules for building edits:**
- `old_string` must be the EXACT line content from `rich.context.lines[N]`
- `new_string` must fix exactly the reported issue and nothing else
- Never change more than one line per fix (multi-line fixes get separate fix entries)
- Verify mentally: does the `new_string` have the `|` at `expected_cols`?

### 5. Write the plan

Output a valid `plan.json` matching this schema:

```json
{
  "schema_version": "1",
  "generated_by": "fix-guide",
  "source_report": "<path to rich.json>",
  "summary": {
    "total_fixes": <N>,
    "high_confidence": <N>,
    "medium_confidence": <N>,
    "low_confidence": <N>,
    "files_affected": <N>
  },
  "fixes": [
    {
      "id": "fix-001",
      "file": "<relative path>",
      "description": "<one sentence: what's wrong and how it's fixed>",
      "confidence": "high",
      "reasoning": "<why this fix is correct — column arithmetic, char count>",
      "diagnostic": {
        "code": "<ascii_box_col etc>",
        "line": <N>,
        "col": <N>
      },
      "edit": {
        "line": <N>,
        "old_string": "<exact line>",
        "new_string": "<fixed line>"
      }
    }
  ]
}
```

### 6. Verify the plan before writing

Before writing plan.json, do a final sanity pass:
- Count chars in old_string — does it match the reported width?
- Count chars in new_string — does it now match the expected width?
- For col fixes: do the `|` positions in new_string match `expected_cols`?
- Does old_string appear in the file (search the lines context)?

### 7. Write plan.json

Use the Write tool to write the plan file.

### 8. Report summary

After writing, summarize:
- Files affected
- Total fixes: N high, M medium, K low
- Any diagnostics skipped (low confidence, structural issues)
- Command to preview: `proof fix --plan plan.json --dry-run`
- Command to apply:  `proof fix --plan plan.json --min-confidence high`

## Edge Cases

**A file has 50 similar errors (e.g., all cells missing padding):**
Generate all 50 fixes. They're all high confidence. The plan handles bulk application.

**A box needs structural redesign (too many columns off):**
Set confidence: low. Describe what's wrong. Author redesigns the box.

**The context lines don't include the failing line:**
The `lines` map may be clipped. Read the actual file to get the exact line content.
Use the Read tool on the file path + the diagnostic line number.

**The old_string must match exactly — trailing spaces matter:**
If a line has a trailing space, include it in old_string. proof fix skips mismatches.
