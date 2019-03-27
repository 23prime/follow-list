#!/bin/sh -eu

RUN="cargo run --release"
${RUN} &

while true
do
isAlive=`ps -e | grep "follow-list" | grep -v grep | wc -l`
if [ $isAlive -eq 0 ]; then
    # echo "Process is dead."
    ${RUN} &
else
    # echo "Process is alive."
    :
fi
sleep 60
done



