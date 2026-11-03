#!/usr/bin/env bash
for i in 1 2 3; do
  printf '\e[?2026h\e[2K\r\e[33mframe %s\e[0m' "$i"
  sleep 0.05
  printf ' drawn whole\e[?2026l'
  sleep 0.15
done
printf '\n'
