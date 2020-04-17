.PHONY:

infra:
	pushd terraform/apply \
	&& terraform apply && popd

destroy:
	pushd terraform/apply \
	&& terraform destroy && popd

ami:
	packer build ./build/http_handlers.json 
