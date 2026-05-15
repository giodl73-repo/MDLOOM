<!-- proof:compiled from="proof:slides" count=2 -->
```slides
SLIDE 1 ─────────────────────────────────────────────────────────────────────── 1/2
Eisenhower matrix


────────────────────────────────────────────────────────────────────────────────
 URGENT + IMPORTANT                      NOT URGENT + IMPORTANT
 - Production incidents                  - Architecture migrations
 - Spec breakage from upstream           - Test coverage gaps

I
m
p
o
r
t───────────────────────────────────────────────────────────────────────────────
aURGENT + NOT IMPORTANT                  NOT URGENT + NOT IMPORTANT
n- "Quick" tickets that aren't quick     - Bikeshed discussions
c- Stand-up status threads               - Premature tooling polish
e





                                    Urgency
SLIDE 2 ─────────────────────────────────────────────────────────────────────── 2/2
Build vs buy


────────────────────────────────────────────────────────────────────────────────
 HIGH SPECIFICITY + LOW COST             HIGH SPECIFICITY + HIGH COST
 Build it ourselves; this is the sweet   Build with caution. Spike a prototype
 spot.                                   first.

S
p
e
c
i
f───────────────────────────────────────────────────────────────────────────────
iLOW SPECIFICITY + LOW COST              LOW SPECIFICITY + HIGH COST
cBuy or use OSS. Engineering time is moreBuy. Custom-building common
ivaluable.                               infrastructure is rarely the right
t                                        call.
y




                                      Cost
```
<!-- /proof:compiled -->
