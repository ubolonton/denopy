# DenoPy (WIP)

Bare-minimum binding of `deno_core`, for embedding JavaScript in Python.

Example:
```python
import denopy
r = denopy.Runtime()
r.eval("['1', '2', '3'].map(parseInt)")
```
