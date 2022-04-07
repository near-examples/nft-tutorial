#!/bin/bash

if [[ $(arch) == 'arm64' ]]; then
  exec arch -x86_64 zsh -l
fi
