
## locations-rs

Rust program for finding all Google Maps place IDs given a zoom level and tile radius. This can be combined with [loc-metadata-rs](../location-metadata-rs) (WIP) to find the place name, rating etc. for every Google Maps location that exists.

An example place ID is `0d134e199a405a163` for the [Empire State Building](https://www.google.com/maps/place/data=!3m2!4b1!5s0x8b398fecd1aea119:0x76fa1e3ac5a94c70!4m6!3m5!1s0x89c259a9b3117469:0xd134e199a405a163!8m2!3d40.7484405!4d-73.9856644!16zL20vMDJuZF8).

### How it works

In Google Maps, area is represented by tiles given a specific zoom level (how far you zoom into [Google Maps](https://maps.google.com)). Each tile consists of an x & y value. The more you zoom in, the more tiles are rendered and more places show up. More popular places would show up in smaller zoom levels while less popular places would require a larger zoom level to show up.

This program utilizes a reverse-engineered Google Maps API endpoint to be able to bulk query tiles given a zoom level. For more information on how this endpoint works, you can see the [maps crate implementation](../maps/src/tiles/tiles.rs) of this endpoint.

The following table describes the available zoom levels and the max X/Y value. The max X/Y value can be calculated as $2^{\text{zoom}} - 1$


| Zoom | Max X/Y | Total Tiles |
| ---- | ------- | ----------- |
| 22 | 4194303 | 17 trillion |
| 21 | 2097151 | 4 trillion |
| 20 | 1048575 | 1 trillion |
| 19 | 524287 | 274 billion |
| 18 | 262143 | 68 billion |
| 17 | 131071 | 17 billion |
| 16 | 65535 | 4 billion |
| 15 | 32767 | 1 billion |
| 14 | 16383 | 268 million |
| 13 | 8191 | 67 million |

### Installation (linux/wsl)

The installation requires [cargo](https://rustup.rs/) to be installed for compilation.

Clone the repository
```
git clone https://github.com/ddd/google_maps
```

Move to the locations-rs folder

```
cd google_maps/locations-rs
```

Modify the `config.yaml` file accordingly. By default, it finds location IDs for the zoom level 16.

```bash
vim config.yaml
```

Compile and run the program. Depending on the number of fetchers, it's [recommended to increase](https://access.redhat.com/solutions/61334) your `ulimit -n` (number of open file descriptors) to unlimited.

```
cargo run --release
```

You can find the locations found in `output.csv`

### ScyllaDB integration (optional)

This program also comes with scylladb integration. To use this, simply set output to `database` in [config.yaml](./config.yaml) and set the database uri and keyspace accordingly.

It requires the following table setup on the keyspace:

```cql
CREATE TABLE google_maps.locations (
    location_id text PRIMARY KEY
);
```