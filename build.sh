#!/usr/bin/env bash
docker buildx build --platform=linux/arm64,linux/amd64,linux/s390x .