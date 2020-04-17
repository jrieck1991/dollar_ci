.PHONY:

infra:
	pushd terraform/apply \
	&& terraform apply && popd

ami:
	packer build ./build/http_handlers.json 
