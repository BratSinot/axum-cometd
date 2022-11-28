# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## [0.3.1]
### Fixed

### Add

### Change

## [0.3.0]
### Fixed
- Fix possible deadlock at unsubscribe. 

### Add

### Change
- `LongPoolingServiceContext::send` return `Result` if message wasn't send.

## [0.2.0] - 2022-11-23
### Fixed

### Add
- Add `LongPoolingServiceContextBuilder`.
- Add `RouterBuilder`.

### Change
- Remove `LongPoolingServiceContext::new()`.
- Remove `LongPoolingServiceContext::build()`.
- Skip serializing `Option::None`.

## [0.1.1] - 2022-11-22
### Fixed

### Add

### Change
- Use `self: &Arc<Self>` instead of `LongPoolingServiceContext(Arc<InnerLongPoolingServiceContext>)` ([#5](https://github.com/BratSinot/axum-cometd/pull/5))

## [0.1.0] - 2022-11-22
- Initial release.
