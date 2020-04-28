# dollar_ci

cheap ci/cd

## compile

Rust code is cross-compiled from Mac OSX to `x86_64-unknown-linux-musl`, you will need the musl-cross toolchain to compile: `brew install FiloSottile/musl-cross/musl-cross` (It can take a while to install)

## packer

EC2 AMI's are built in the default VPC of `us-east-1`. To recreate a default vpc run: `aws ec2 create-default-vpc`. The packer subnet field is hardcoded to an existing subnet in a default VPC, update this value if the default VPC changes.

## terraform

All infrastructure is managed by terraform, backend will be migrated to an S3 bucket.

## TODO

* a way to clean up old amis + snapshots
* TLS
* billing alerts
* s3 bucket for terraform backups
* nginx with eip for ingress
* get all github payloads in json for unit tests
* build http client more efficiently
* fix terraform VPC race condition
* read github headers
* add the github headers we need
* rename HandlersErr
* rename NotFound to something about validation failed
* figure out how to only call get installation token once per invocation of the github client
