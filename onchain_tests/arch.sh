#!/bin/bash

if [[ $(arch) == 'arm64' ]]; then
  arch -x86_64 zsh
fi