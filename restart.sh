#!/usr/bin/env sh

launchctl unload ~/Library/LaunchAgents/ckampfe.clipurl.plist
launchctl load -w ~/Library/LaunchAgents/ckampfe.clipurl.plist
