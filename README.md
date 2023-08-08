# discogs-rs

Web scraping command-line tool for [Discogs](https://www.discogs.com/).

- Check items from you wantlist, select sellers, add to cart.

- Add items to your wantlist (LP only).

This is a hobby project, and I'm not planning to introduce new features anytime
soon.

## Usage

Export the cookies from a session in Discogs using some tool like
[Cookie-editor] (https://cookie-editor.cgagnier.ca/), and save them in
a `.cookies.json` file. Use the path to the file in `<COOKIES>`.

Use `-w --wantlist` if you wish to add LPs to your wantlist. Just use the name
of the album and then select the master release. LPs will be added
automatically.

```shell 

Usage: discogs [OPTIONS] <COOKIES>

Arguments:
  <COOKIES>

Options:
  -w, --wantlist <WANTLIST>  [default: ]
  -h, --help                 Print help
  -V, --version              Print version

```
