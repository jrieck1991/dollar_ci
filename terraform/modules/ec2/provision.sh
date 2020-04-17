#!/bin/bash

systemctl restart amazon-ssm-agent

# start http server
./home/ec2-user/http_handlers
