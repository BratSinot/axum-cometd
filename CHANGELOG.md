# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## [0.9.0]

### Fixed

### Add

- Additional data for callbacks through `axum::Extension`.

### Change

- `LongPollingServiceContext` now generic.
- Use channel instead of callbacks.
- Make `SendError` copy.

## [0.8.0]

### Fixed

### Add

### Change

- Callbacks now accept reference to shared pointer AND arguments.

## [0.7.1]

### Fixed

### Add

### Change

- Adjust logging.

## [0.7.0]

### Fixed

### Add

- Add subscribeAdded callback.
- Add `BAYEUX_BROWSER` cookie.

### Change

- Replace tuple by struct in callbacks.

## [0.6.1]

### Fixed

### Add

### Change
- Use rotr for generating next ClientId.

## [0.6.0]

### Fixed

- Fix subscribing to multiple channels.
- Fix wrong channel name in client message.
- Fix return `408 REQUEST_TIMEOUT` instead of advice reconnect.
- Missing messages for client id. Switch to `tokio::sync::mpsc`.
  Parallel connection with same `clientId` give error.

### Add

- Wildcards support.
- Channel name validation.
- Validate channel on `send` and `send_to_client`.

### Change

- `SendError` now enum.

## [0.5.0] - 2022-12-06

### Fixed

- Check channel before start wait for client message.
- Support `subscribe` field be single-value or array.

### Add

### Change

## [0.4.1] - 2022-12-05

### Fixed

### Add

- `LongPoolingServiceContext::send_to_client` to send message directly to client.
- `LongPoolingServiceContext::unsubscribe` to remove client.

### Change

- Return BAD_REQUEST if client send publish and connect at one request.

## [0.4.0] - 2022-12-05

### Fixed

### Add

- Add support for client message publishing.

- Add session added / removed callbacks.

### Change

- Remove `Msg` generic from `LongPoolingServiceContext`. `LongPoolingServiceContext::send` is generic now.
- Sync some responses with JS version.

## [0.3.0] - 2022-11-30

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
