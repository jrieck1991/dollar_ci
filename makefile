.PHONY: infra destroy ami

infra:
	pushd terraform/apply \
	&& terraform init \
	&& terraform apply && popd

destroy:
	pushd terraform/apply \
	&& terraform destroy && popd

ami: http_handlers
	PACKER_LOG=1 packer build ./build/http_handlers.json 

compile:
	pushd http_handlers \
	&& cargo build --release --target=x86_64-unknown-linux-musl && popd
