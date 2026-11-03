#!/usr/bin/env bash
s='⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'
for i in $(seq 0 23); do printf '\r\e[36m%s\e[0m Installing packages' "${s:i%10:1}"; sleep 0.08; done
printf '\r\e[32m✔\e[0m Installed 42 packages\n'
for p in 0 10 25 40 55 70 85 100; do
  n=$((p/5)); bar=''; rest=''
  for ((k=0;k<n;k++)); do bar+='█'; done
  for ((k=n;k<20;k++)); do rest+='░'; done
  printf '\r\e[35m%s\e[2m%s\e[0m %3d%%' "$bar" "$rest" "$p"; sleep 0.12
done
printf '\n'
