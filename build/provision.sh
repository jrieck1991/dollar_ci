#!/bin/bash

# move systemd service file to correct directory, we need sudo for this
sudo mv /tmp/http_handlers.service /etc/systemd/system/http_handlers.service

# start on boot
sudo systemctl enable http_handlers
