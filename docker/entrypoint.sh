#!/bin/sh

if [ "$ONLY_SERVER" = "1" ]; then
  echo "[entrypoint.sh] ONLY_SERVER is set. Skipping Nginx."
  supervisord -c /etc/supervisor/supervisord.only_server.conf
else
  supervisord -c /etc/supervisor/supervisord.conf
fi
