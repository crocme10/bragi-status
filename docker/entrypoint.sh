#!/bin/sh

echo "Waiting for bragi..."

while ! nc -z bragi 4000; do
  sleep 0.1
done

echo "Bragi started"
echo "Settings: ${SETTINGS}"

./service run -c /etc/opt/bragi-status -s ${SETTINGS}
