
<a name="v0.7.0"></a>
## [v0.7.0](https://github.com/rsocket/rsocket-rust/compare/v0.6.0...v0.7.0) (2021-01-14)

### Chore

* update deps ([#42](https://github.com/rsocket/rsocket-rust/issues/42))
* config chglog ([#41](https://github.com/rsocket/rsocket-rust/issues/41))
* use mkcert to generate TLS example certificates and keys ([#38](https://github.com/rsocket/rsocket-rust/issues/38))
* add RSocket trait example in readme
* fix readme
* bump tokio to v0.3.6
* **rustfmt:** optimize import ([#39](https://github.com/rsocket/rsocket-rust/issues/39))

### Feat

* close connection correctly when client is dropped ([#40](https://github.com/rsocket/rsocket-rust/issues/40))
* migrate to tokio v1
* change transport api ([#35](https://github.com/rsocket/rsocket-rust/issues/35))
* implment tls transport ([#31](https://github.com/rsocket/rsocket-rust/issues/31))
* redesign RSocket trait based on async_trait
* **request_response:** handle empty response correctly

### Fix

* register client-side responder correctly ([#36](https://github.com/rsocket/rsocket-rust/issues/36))
* simplify Option convert
* remove useless examples


<a name="v0.6.0"></a>
## [v0.6.0](https://github.com/rsocket/rsocket-rust/compare/v0.5.3...v0.6.0) (2020-12-13)

### Chore

* prelease 0.6
* upgrade deps
* use gh actions instead of travis ([#22](https://github.com/rsocket/rsocket-rust/issues/22))

### Feat

* implement client-side keepalive ([#25](https://github.com/rsocket/rsocket-rust/issues/25))
* support tokio v0.3.x ([#23](https://github.com/rsocket/rsocket-rust/issues/23))

### Fix

* convert bytes to utf8 safely ([#27](https://github.com/rsocket/rsocket-rust/issues/27))
* optimize bytes write action ([#24](https://github.com/rsocket/rsocket-rust/issues/24))

### Refactor

* use thiserror & anyhow as error struct ([#20](https://github.com/rsocket/rsocket-rust/issues/20))

### Pull Requests

* Merge pull request [#19](https://github.com/rsocket/rsocket-rust/issues/19) from rsocket/develop


<a name="v0.5.3"></a>
## [v0.5.3](https://github.com/rsocket/rsocket-rust/compare/v0.5.2...v0.5.3) (2020-06-11)

### Pull Requests

* Merge pull request [#17](https://github.com/rsocket/rsocket-rust/issues/17) from rsocket/develop
* Merge pull request [#16](https://github.com/rsocket/rsocket-rust/issues/16) from seal90/develop


<a name="v0.5.2"></a>
## [v0.5.2](https://github.com/rsocket/rsocket-rust/compare/v0.5.1...v0.5.2) (2020-05-26)

### Pull Requests

* Merge pull request [#14](https://github.com/rsocket/rsocket-rust/issues/14) from rsocket/feature/messaging
* Merge pull request [#11](https://github.com/rsocket/rsocket-rust/issues/11) from kuronyago/feature/wasm_fire_and_forget
* Merge pull request [#9](https://github.com/rsocket/rsocket-rust/issues/9) from kuronyago/docs/readme_for_websocket_example


<a name="v0.5.1"></a>
## [v0.5.1](https://github.com/rsocket/rsocket-rust/compare/v0.5.0...v0.5.1) (2020-04-06)

### Pull Requests

* Merge pull request [#8](https://github.com/rsocket/rsocket-rust/issues/8) from rsocket/develop


<a name="v0.5.0"></a>
## [v0.5.0](https://github.com/rsocket/rsocket-rust/compare/v0.4.0...v0.5.0) (2020-02-22)

### Pull Requests

* Merge pull request [#6](https://github.com/rsocket/rsocket-rust/issues/6) from rsocket/improve/pick_transport


<a name="v0.4.0"></a>
## [v0.4.0](https://github.com/rsocket/rsocket-rust/compare/v0.3.0...v0.4.0) (2019-12-24)

### Bugfix

* response payload of REQUEST_RESPONSE will be sent with NEXT|COMPLETE flag.

### Pull Requests

* Merge pull request [#4](https://github.com/rsocket/rsocket-rust/issues/4) from rsocket/develop


<a name="v0.3.0"></a>
## [v0.3.0](https://github.com/rsocket/rsocket-rust/compare/v0.2.0...v0.3.0) (2019-12-04)

### Pull Requests

* Merge pull request [#3](https://github.com/rsocket/rsocket-rust/issues/3) from rsocket/feature/routing_metadata


<a name="v0.2.0"></a>
## [v0.2.0](https://github.com/rsocket/rsocket-rust/compare/v0.1.5...v0.2.0) (2019-11-29)

### Pull Requests

* Merge pull request [#2](https://github.com/rsocket/rsocket-rust/issues/2) from rsocket/feature/async_await


<a name="v0.1.5"></a>
## [v0.1.5](https://github.com/rsocket/rsocket-rust/compare/v0.1.4...v0.1.5) (2019-10-08)


<a name="v0.1.4"></a>
## [v0.1.4](https://github.com/rsocket/rsocket-rust/compare/v0.1.3...v0.1.4) (2019-09-06)


<a name="v0.1.3"></a>
## [v0.1.3](https://github.com/rsocket/rsocket-rust/compare/v0.1.2...v0.1.3) (2019-09-03)


<a name="v0.1.2"></a>
## [v0.1.2](https://github.com/rsocket/rsocket-rust/compare/v0.1.0...v0.1.2) (2019-09-02)


<a name="v0.1.0"></a>
## v0.1.0 (2019-08-29)

