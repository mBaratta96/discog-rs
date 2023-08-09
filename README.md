# discogs-rs

Web scraping command-line tool for [Discogs](https://www.discogs.com/).

- Check items from you wantlist, select sellers, add to cart.

- Add items to your wantlist (LP only).

- Check your cart.

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

Usage: discogs <COOKIES> <COMMAND>

Commands:
  add       
  cart      
  wantlist  
  help      Print this message or the help of the given subcommand(s)

Arguments:
  <COOKIES>  

Options:
  -h, --help     Print help
  -V, --version  Print version

```
