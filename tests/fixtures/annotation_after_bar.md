# Annotation After Bar — Should Report Width Error

A content row with text appended after the closing `|`. The author intended it as
an inline comment but it makes the row wider than the box.

```
+------------------+
| normal content   |
| content with ann |  <- this annotation makes the row too wide
+------------------+
```

A correct version has the annotation on a separate line:

```
+------------------+
| normal content   |
| annotated item   |
+------------------+
```
Note: The annotation goes here, outside the code block.
