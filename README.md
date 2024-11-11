# Sugoi

small web server for waking up and putting my server to sleep.

## Developing

To compile for Raspberry Pi Zero W:
```sh
cross build --target arm-unknown-linux-gnueabihf --release
```

## TODO

- get it working with Nix (add package and service possibly)
- use tailwind CLI to generate CSS
- make table scrollable after certain amount of rows in the body
- implement my own ring buffer to hold the status data (the current ones don't meet some of my requirements from skimming their docs)
- separate header and table components into separate files for easier reading
- add a root page as well
