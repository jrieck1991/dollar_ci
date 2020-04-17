# dollar_ci

cheap ci/cd

## packer

Images are built in the default VPC of `us-east-1`. To recreate a default vpc run: `aws ec2 create-default-vpc`

## TODO

* deploy rust http_handlers to autoscaling ec2 instance
* a way to clean up old amis + snapshots
