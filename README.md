# hub-util
A CLI tool written in Rust for interacting with Blackmagic Videohub devices

## Usage
### Creating a dump
To create a dump file use the `dump` command and specify the
ip address of the Videohub device. By default, if a port is not provided the
default port of 9990 will be used.

```
./hub_util dump --ip <ip address> > dump.json
```

### Importing a dump
Once you have created a dump file using the dump command you can transfer all
parameters to another Videohub device using the `import` command.
```
./hub_util import --ip <ip address> --file dump.json
```

## Dump file schema
The schema for the dump file can be found in the [schema.json](schema.json) file.

An example file looks like this:
```json5
{
  "timestamp": 1741618011000,
  "name": "Smart Videohub 20 x 20",
  "sources": [
    {
      "id": 1,
      "name": "Destination 1"
    },
    {
      "id": 2,
      "name": "Destination 2"
    }
    // omitted for brevity...
  ],
  "destinations": [
    {
      "id": 1,
      "name": "Destination 1"
    },
    {
      "id": 2,
      "name": "Destination 2"
    }
    // omitted for brevity...
  ],
  "routes": [
    // an example 1-1 routing
    {
      "destinationId": 1,
      "routeId": 1,
    },
    {
      "destinationId": 2,
      "routeId": 2,
    }
    // omitted for brevity...
  ]
}
```

# Building
The project can be built using 
```
cargo build
```