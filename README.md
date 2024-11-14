# Sugoi

small web server for waking up and putting my server to sleep.

## Developing

To compile for Raspberry Pi Zero W:
```sh
cross build --target arm-unknown-linux-gnueabihf --release
```

## TODO

- separate header and table components into separate files for easier reading
- get it working with Nix (add package and service possibly)
- make table have pagination (scrollable seems complicated)
- implement my own ring buffer to hold the status data (the current ones don't meet some of my requirements from skimming their docs)
- add a root page as well
