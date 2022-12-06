# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## [0.4.2]

### Fixed

- Check channel before start wait for client message.

### Add

### Change

## [0.4.1]

### Fixed

### Add

- `LongPoolingServiceContext::send_to_client` to send message directly to client.
- `LongPoolingServiceContext::unsubscribe` to remove client.

### Change

- Return BAD_REQUEST if client send publish and connect at one request.

## [0.4.0]

### Fixed

### Add

- Add support for client message publishing.

- Add session added / removed callbacks.

### Change

- Remove `Msg` generic from `LongPoolingServiceContext`. `LongPoolingServiceContext::send` is generic now.
- Sync some responses with JS version.

## [0.3.0]

### Fixed

- Fix possible deadlock at unsubscribe.

### Add

- `LongPoolingServiceContextBuilder::client_storage_capacity`
- `LongPoolingServiceContextBuilder::subscription_storage_capacity`

### Change

- `LongPoolingServiceContext::send` return `Result` if message wasn't send.
- Update `axum` to `0.6.x`.
- Remove `RouterBuilder::base_path`.
- Switch to `async-broadcast` crate for client pub-sub.

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

- Use `self: &Arc<Self>` instead
  of `LongPoolingServiceContext(Arc<InnerLongPoolingServiceContext>)` ([#5](https://github.com/BratSinot/axum-cometd/pull/5))

## [0.1.0] - 2022-11-22

- Initial release.
