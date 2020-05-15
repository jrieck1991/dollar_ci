# dollar_ci

Cheap ci/cd with encrypted builds running on a decentralized network.

## compile

Rust code is cross-compiled from Mac OSX to `x86_64-unknown-linux-musl`, you will need the musl-cross toolchain to compile: `brew install FiloSottile/musl-cross/musl-cross` (It can take a while to install)

## packer

EC2 AMI's are built in the default VPC of `us-east-1`. To recreate a default vpc run: `aws ec2 create-default-vpc`. The packer subnet field is hardcoded to an existing subnet in a default VPC, update this value if the default VPC changes.
