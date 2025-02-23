# The Bonus Points List Visualizer

The Bonus Points is a podcast in which Oli and Joe play a new game every two weeks, and rank it on their personal list. All the data they have produced by doing this begs to be analyzed, hence why this project exists.

All of the generated images can be found [here](https://w-graj.net/images/tgl-viz/).

![IMAGE](https://w-graj.net/images/tgl-viz/list_over_time.png)

## Usage

Create a `.env` file containing credentials for the twitch API, as detailed [here](https://api-docs.igdb.com/#getting-started).
```sh
CLIENT_ID=...
CLIENT_SECRET=...
```

Then simply
```sh
cargo run --release
```

## License

```
Copyright (C) 2025  Wojciech Graj

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```
