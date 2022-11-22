# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## [Unreleased]
### Added
- Added `LongPoolingServiceContextBuilder`.

### Changed
- Remove `LongPoolingServiceContext::new()`.

## [0.1.1] - 2022-11-22
### Added

### Changed
- Use `self: &Arc<Self>` instead of `LongPoolingServiceContext(Arc<InnerLongPoolingServiceContext>)` ([#5](https://github.com/BratSinot/axum-cometd/pull/5))

## [0.1.0] - 2022-11-22
- Initial release.
