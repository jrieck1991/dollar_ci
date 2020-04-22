.PHONY: infra destroy ami

## AWS targets expect 'AWS_PROFILE' to be set

apply:
	pushd terraform/apply \
	&& terraform init \
	&& terraform apply && popd

destroy:
	pushd terraform/apply \
	&& terraform init \
	&& terraform destroy && popd

# create new ami
ami: compile
	PACKER_LOG=1 packer build ./build/http_handlers.json 

# compile rust code for musl
compile:
	pushd http_handlers \
	&& OPENSSL_LIB_DIR=/usr/local/opt/openssl/lib \
	CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc \
	cargo build --release --target=x86_64-unknown-linux-musl && popd
