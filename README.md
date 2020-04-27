# dollar_ci

cheap ci/cd

## compile

Rust code is cross-compiled from Mac OSX to `x86_64-unknown-linux-musl`, you will need the musl-cross toolchain to compile: `brew install FiloSottile/musl-cross/musl-cross` (It can take a while to install)

## packer

Images are built in the default VPC of `us-east-1`. To recreate a default vpc run: `aws ec2 create-default-vpc`

## TODO

* a way to clean up old amis + snapshots
* TLS
* billing alerts
* s3 bucket for terraform backups
* nginx with eip for ingress
* get all github payloads in json for unit tests
* build http client more efficiently
* separate code into different files based on module?
* fix terraform VPC race condition
