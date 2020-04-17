#!/bin/bash

# restart agent to pickup config applied by terraform
systemctl restart amazon-ssm-agent

# start http server
./home/ec2-user/http_handlers
