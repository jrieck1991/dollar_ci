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

ami: compile
	PACKER_LOG=1 packer build ./build/http_handlers.json 

# compile rust code for linux
compile:
	pushd http_handlers \
	&& cargo build --release --target=x86_64-unknown-linux-musl && popd
