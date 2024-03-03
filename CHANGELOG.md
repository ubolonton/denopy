# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

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
