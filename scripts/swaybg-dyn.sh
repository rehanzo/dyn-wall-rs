#!/bin/sh
PID=`pidof swaybg`
swaybg -o "*" -i "$1" -m fill &
sleep 1
kill $PID
