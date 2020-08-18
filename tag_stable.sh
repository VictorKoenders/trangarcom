#!/bin/bash
set -ex

docker tag trangarcom:latest trangar.azurecr.io/trangarcom:stable
az acr login -n trangar
docker push trangar.azurecr.io/trangarcom:stable