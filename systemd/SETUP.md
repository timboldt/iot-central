# Instructions

## Initial setup

1. Copy `env.txt` and all the binaries to `~/bin/`.
2. Populate `env.txt` with your secrets.
3. Copy the binaries to `~/bin/`.
4. Copy `*.service` to `/lib/systemd/system/`.
5. `sudo systemctl daemon-reload`
6. `sudo systemctl start your-service.service`

# Updating just the binaries

1. Copy the binaries to `~/bin/`.
2. `sudo systemctl restart your-service.service`
