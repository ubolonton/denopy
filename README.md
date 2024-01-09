# DenoPy (WIP)

Bare-minimum binding of `deno_core`, for embedding JavaScript in Python.

Example:
```python
import denopy
r = denopy.Runtime()
r.eval("['1', '2', '3'].map(parseInt)")
```

## Notes

- This supports only blocking JavaScript, not `async/await`.
    - Most JavaScript embedding use cases I've seen so far involve pure logic, not I/O.
    - Juggling `async/await` across 3 languages is a lot of work.
- Deno has multiple layers, in decreasing order of functionalities: `cli` -> `deno_runtime` -> `deno_core` -> `v8`. The first 2 are not stable, so we bind the 3rd one. This means:
    - No TypeScript.
    - No [npm module import](https://docs.deno.com/runtime/manual/node).
