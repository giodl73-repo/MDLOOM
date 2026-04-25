# Cell Padding — Should Warn on Missing Padding

Missing left padding (text flush against delimiter):

```
+----------+----------+
|no padding| fine     |
| fine     |no right  |
+----------+----------+
```

Correct padding (1 space each side):

```
+----------+----------+
| fine     | fine     |
| fine     | fine     |
+----------+----------+
```
