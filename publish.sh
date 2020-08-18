#!/bin/bash

set -ex

docker build . -t trangarcom:latest
docker tag trangarcom:latest trangar.azurecr.io/trangarcom:latest
az acr login -n trangar
docker push trangar.azurecr.io/trangarcom:latest
