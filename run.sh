#!/usr/bin/env bash

HERE=$(dirname $(readlink -f $0))
SCREEN_SIZE=${SCREEN_SIZE:-1024x720}
XDISPLAY=${XDISPLAY:-:1}

Xephyr +extension RANDR -screen ${SCREEN_SIZE} ${XDISPLAY} -ac &
XEPHYR_PID=$!
(
	sleep 1
	env DISPLAY=${XDISPLAY} cargo run --example config &
	QTILE_PID=$!
	env DISPLAY=${XDISPLAY} &
	wait $QTILE_PID
	kill $XEPHYR_PID
)
