# Sugoi

small web server for waking up and putting my server to sleep.

## Developing

To compile for Raspberry Pi Zero W:
```sh
cross build --target arm-unknown-linux-gnueabihf --release
```

Commands to populate the table and test api endpoints:
```sh
# Wake Command
curl http://127.0.0.1:8080/api/wake -d "mac_address=11:22:33:44:55:66"

# Sleep Command
curl http://localhost:8080/api/sleep -d "address=localhost:8253"
```

## TODO

- add input forms for sending wake and sleep requests through the UI
- add a root page as well (redirect for now)
- add option to select amount of items displayed in table (pagination.per_page/rows)
- add all colors to a tailwind theme (so I don't need to search all over the place)
- add Nix service
