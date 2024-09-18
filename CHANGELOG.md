# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]
### Changed
- The behavior of whether a JavaScript number is converted into Python `float` or `int` has changed.
    - Before: Numbers that are valid 32-bit signed/unsigned integers (`i32`/`u32`) are converted into `int`. Others are converted into `float`.
    - After: Non-whole numbers are converted into `float`. Whole-number conversion is controlled by the parameter `integer_conversion`. The valid values are, in the order of increasing aggressiveness:
        - `never`: All numbers are converted into `float`.
        - `i32`: Only valid 32-bit integers are converted into `int`. This is the default, for consistency with other common embedded JavaScript engines.
        - `safe`: Whole numbers within the [safe-integer range](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isSafeInteger) are converted into `int`. This is arguably the nicer behavior.
        - `aggressive`: Even [unsafe integers](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER) are converted into `int`.

## [0.4.0] - 2024-03-03
### Added
- JavaScript evaluation raises `JsError` for uncaught JavaScript exceptions. Evalution includes `JsFunction` calls, and `Runtime` methods `eval`, `mod_evaluate`, `call`.
    - For function calls, the raised `JsError` object stores the thrown JavaScript exception in the attribute `value`.
- `Runtime.eval()` accepts an optional `name` argument, for better source code location reporting.

## [0.3.0] - 2024-02-19
### Changed
- JavaScript values of complex types are no longer converted to Python `dict`/`list` by default, but wrapped in `JsObject`/`JsArray` objects.
    - They can be recursively unwrapped with `Runtime.unwrap()`.
    - Methods that return a JavaScript value gain the keyword argument `unwrap`.

### Added
- `JsFunction` objects are now callable directly, without the need to use `Runtime.call()`.
    - They can be called as methods, by passing a keyword argument `this`.
- `Runtime.get(object, property, unwrap=False)`.

## [0.2.0] - 2024-02-02
### Fixed
- Create at most 1 `Runtime` per thread, to fix the segfault when too many `Runtime` objects are created on the stack.

## [0.1.0] - 2024-01-19
Initial release: module loading, code evaluation, type conversions, function calls.

[Unreleased]: https://github.com/ubolonton/denopy/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/ubolonton/denopy/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ubolonton/denopy/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ubolonton/denopy/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ubolonton/denopy/compare/6d975ef1...v0.1.0
