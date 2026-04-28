<!-- proof:compiled from="proof:slides" count=25 -->
```slides
SLIDE 1 ─────────────────────────────────────────────────────────────────────── 1/25
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                  proof Slides                                  
                      ASCII presentations with proof:slide                      
                                                                                
                                  proof guide                                   
                                      2026                                      
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 2 ─────────────────────────────────────────────────────────────────────── 2/25
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                              ── Slide Layouts ──                               
                                                                                
                              Six built-in layouts                              
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 3 ─────────────────────────────────────────────────────────────────────── 3/25
title-content                                                                   
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 4 ─────────────────────────────────────────────────────────────────────── 4/25
                                                                                
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
The most common layout. One title zone at the top,                              
one body zone below. The body accepts any proof: directives.                    
                                                                                
● Clean separation between title and content                                    
● Body supports proof:bullets, proof:callout, proof:divider                     
● Inline $\alpha$, $\beta$ math works in body text                              
● [sym:checkmark] Symbol expansion works too                                    
● ```                                                                           
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 5 ─────────────────────────────────────────────────────────────────────── 5/25
two-column                                                                      
────────────────────────────────────────────────────────────────────────────────
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 6 ─────────────────────────────────────────────────────────────────────── 6/25
                                                                                
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
LEFT COLUMN                                                                     
                                                                                
● Left zone content                                                             
● Use for comparisons                                                           
● Or before/after                                                               
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 7 ─────────────────────────────────────────────────────────────────────── 7/25
                                                                                
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
RIGHT COLUMN                                                                    
                                                                                
● Right zone content                                                            
● Same height as left                                                           
● Ratio is configurable                                                         
● ```                                                                           
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 8 ─────────────────────────────────────────────────────────────────────── 8/25
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                             ── Body Directives ──                              
                                                                                
          proof:bullets · proof:callout · proof:divider · proof:quote           
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 9 ─────────────────────────────────────────────────────────────────────── 9/25
proof:bullets                                                                   
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 10 ────────────────────────────────────────────────────────────────────── 10/25
                                                                                
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
● First level bullet                                                            
  ◦ Nested level two                                                            
    ▸ Level three nesting                                                       
● Back to level one                                                             
● [sym:checkmark] Symbols in bullets                                            
● Math in bullets: $E = mc^2$                                                   
● Wide content wraps at slide width                                             
● ```                                                                           
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 11 ────────────────────────────────────────────────────────────────────── 11/25
proof:callout                                                                   
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 12 ────────────────────────────────────────────────────────────────────── 12/25
                                                                                
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
ℹ This is an info callout. Use for tips, notes, and asides.                     
  The callout box is drawn with rounded corners.                                
                                                                                
⚠ This is a warning callout. Use for cautions and gotchas.                      
                                                                                
◆ This is an error callout. Use for critical information.                       
  ```                                                                           
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 13 ────────────────────────────────────────────────────────────────────── 13/25
proof:divider and proof:quote                                                   
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 14 ────────────────────────────────────────────────────────────────────── 14/25
                                                                                
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
────────────────────────────────────────────────────────────────────────────────
                                                                                
               “Premature optimization is the root of all evil.”                
                                 — Donald Knuth                                 
                                                                                
────────────────────────────────────────────────────────────────────────────────
                                                                                
                           Centered text is centered.                           
                                      ```                                       
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 15 ────────────────────────────────────────────────────────────────────── 15/25
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 16 ────────────────────────────────────────────────────────────────────── 16/25
                                                                                
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
proof:stat label="Tests" value="626" delta="+147"                               
proof:stat label="Modules" value="17" delta="+1"                                
proof:stat label="LOC" value="~8,000" delta=""                                  
proof:stat label="Coverage" value="high" delta=""                               
```                                                                             
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 17 ────────────────────────────────────────────────────────────────────── 17/25
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                              ── Math in Slides ──                              
                                                                                
                    Inline $...$ expansion in all text zones                    
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 18 ────────────────────────────────────────────────────────────────────── 18/25
Inline Math                                                                     
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 19 ────────────────────────────────────────────────────────────────────── 19/25
                                                                                
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
Inline math works everywhere in slide body:                                     
                                                                                
α + β = γ — Greek letters expand.                                               
                                                                                
x² + y² = z² — Superscripts render as Unicode.                                  
                                                                                
∀ ε > 0, ∃ δ > 0 — Logic symbols.                                               
                                                                                
∇ × B = μ₀ J — Maxwell's equation.                                              
                                                                                
────────────────────────────────────────────────────────────────────────────────
                                                                                
For multi-line math, use proof:math in a separate document.                     
```                                                                             
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 20 ────────────────────────────────────────────────────────────────────── 20/25
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 21 ────────────────────────────────────────────────────────────────────── 21/25
                                                                                
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
╔═══════════════════════════════════════════╗                                   
      ║                                           ║                             
      ║   proof:slide layout=blank                ║                             
      ║                                           ║                             
      ║   The blank layout gives you a full       ║                             
      ║   canvas — no chrome, no header.          ║                             
      ║   Draw whatever you want.                 ║                             
      ║                                           ║                             
      ╚═══════════════════════════════════════════╝                             
```                                                                             
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 22 ────────────────────────────────────────────────────────────────────── 22/25
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                Slide Attributes                                
                     width · height · theme · show-numbers                      
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 23 ────────────────────────────────────────────────────────────────────── 23/25
proof.toml for Slides                                                           
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 24 ────────────────────────────────────────────────────────────────────── 24/25
                                                                                
                                                                                
                                                                                
────────────────────────────────────────────────────────────────────────────────
Configure slide defaults in proof.toml:                                         
                                                                                
● width: output width in characters (default: 120)                              
● height: output height in lines (default: 34)                                  
● theme: minimal | box | none                                                   
● show-numbers: true | false                                                    
                                                                                
Per-slide overrides go in the fence header:                                     
                                                                                
```proof:slide layout=title width=60 height=15 theme=box                        
title: "Narrow slide"                                                           
```                                                                             
```                                                                             
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
SLIDE 25 ────────────────────────────────────────────────────────────────────── 25/25
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                      End                                       
                 See also: elements.md · math.md · dashboard.md                 
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
```
<!-- /proof:compiled -->
