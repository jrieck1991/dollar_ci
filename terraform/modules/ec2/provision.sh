#!/bin/bash

systemctl restart amazon-ssm-agent

# start http server
./usr/bin/http_handlers
