
# mix_server
[![](https://travis-ci.org/david415/mix_server.png?branch=master)](https://www.travis-ci.org/david415/mix_server) [![](https://img.shields.io/crates/v/mix_server.svg)](https://crates.io/crates/mix_server) [![](https://docs.rs/mix_server/badge.svg)](https://docs.rs/mix_server/)

This rust crate provides a library for writing mixnet servers.


# warning

This code has not been formally audited by a cryptographer or security researcher.
It therefore should not be considered safe or correct. Use it at your own risk!


# details

This mix network library is design for writing mix servers using the
SEDA computational model; Staged Event Driven Architecture allows for
dynamic loading and graceful performance degradation which is are features
any good software router ought to have.


# acknowledgments

Thanks to Yawning Angel for the inspiring design of the Katzenpost mix server library.


# license

GNU AFFERO GENERAL PUBLIC LICENSE