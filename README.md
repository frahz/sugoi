# Sugoi

small web server for waking up and putting my server to sleep.

## Developing

To compile for Raspberry Pi Zero W:
```sh
cross build --target arm-unknown-linux-gnueabihf --release
```

## TODO

- make table have pagination (scrollable seems complicated)
- add input forms for sending wake and sleep requests through the UI
- add a root page as well (redirect for now)
- add all colors to a tailwind theme (so I don't need to search all over the place)
- separate header and table components into separate files for easier reading
- add Nix service (package has been added)
- implement my own ring buffer to hold the status data (the current ones don't meet some of my requirements from skimming their docs)
