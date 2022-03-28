#!/bin/bash

for f in pointercrate-*
do
  (
    cd "$f" && cargo sqlx prepare --check;
  )
done;