#!/bin/bash

set -ex

docker build . -t trangarcom:latest
docker tag trangarcom trangar.azurecr.io/trangarcom
az acr login -n trangar
docker push trangar.azurecr.io/trangarcom
