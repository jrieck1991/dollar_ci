.PHONY: infra destroy ami

infra:
	pushd terraform/apply \
	&& terraform apply && popd

destroy:
	pushd terraform/apply \
	&& terraform destroy && popd

ami:
	PACKER_LOG=1 packer build ./build/http_handlers.json 
