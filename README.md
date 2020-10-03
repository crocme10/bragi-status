# Bragi Status

> Discover the status of bragi (and underlying elasticsearch) endpoint

This program takes an environment name, and return a json output describing the status
of the corresponding bragi instance. The status includes a description of the underlying
elasticsearch cluster, and all its indices.

This program is configured using a file, `env.json`, which contains all entry points (read
elasticsearch backend).

## Installation

OS X & Linux:

```sh
cargo build --release
```

Windows:

??

## Configuration

You need to supply a `env.json` file which describes all the entry points. There is a script,
`env.sh`, which generates the file for you. You need to pass the path to
a `navitian_deployment_conf` directory. The script will look in the files in that directory for
entry point, and gather them on stdout. You can redirect the output to `env.json`:

```
./env.sh [..]/navitian_deployment_conf > env.json
```

produces the following output (`env.json`):

```
[
  {
    "name": "prod",
    "url": "http://vippriv-bragi-ws.mutu.prod.canaltp.prod"
  },
  {
    "name": "dev",
    "url": "http://bragi-ws.ctp.dev.canaltp.fr"
  },
  ...
]
```

## Usage example

The following uses [jq](https://stedolan.github.io/jq/) to format and display the json:

```shell
cargo run dev | jq '.'
{
  "label": "bragi_dev",
  "url": "http://bragi-ws.ctp.dev.canaltp.fr",
  "version": "v1.13.0-44-gbd7d3be-modified",
  "status": "available",
  "updated_at": "2019-12-24T14:35:53.611947258Z",
  "elastic": {
    "label": "elasticsearch_dev",
    "url": "http://vippriv-nav2-dev-es.canaltp.prod:9200",
    "name": "",
    "status": "available",
    "version": "2.4.6",
    "indices": [
      {
        "label": "munin_stop_fr-bre_20191010_182020_368295686",
        "place_type": "stop",
        "coverage": "fr-bre",
        "date": "2019-10-10T18:20:20Z",
        "count": 12635,
        "updated_at": "2019-12-24T14:35:53.875292447Z"
      },
      ...
    ]
  }
}
```

## Development setup

This is rust code, which necessitates a [rust
environment](https://www.rust-lang.org/learn/get-started)

```sh
cargo build --release
./target/release/bragi-status dev
```

## Release History

* 0.1.0
    * Baseline

## Meta

Matthieu Paindavoine  [Area403](http://area403.org) – matt@area403.org

Distributed under the MIT license. See ``LICENSE`` for more information.

[https://github.com/yourname/github-link](https://github.com/dbader/)

## Contributing

1. Fork it (<https://github.com/yourname/yourproject/fork>)
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request
