# home-consule

A (micro) dashboard for your homelab integrated with [Consul](https://www.consul.io/)

## Usage

This project will feed Consul's service catalogue into a [handlebar](https://handlebarsjs.com/guide/) template file.

It will serve the current directiory staticaly, so you can include images and CSS if needed.

A basic `index.hbs` template looks like:

```handlebars
<h1>Welcome to the Homelab</h1>
<img src="/static/header.jpg" />
<ul>
    {{#each this}}
    <li><a href="https://{{this}}.example.com">{{this}}</a></li>
    {{/each}}
</ul>
```

## Run

This project aims to run on Docker with minimal size, and also targets multiplaform builds, including Windows, Linux and Raspberry Pi (Zero to 4).

You can either compile it with `cargo build --release` or use the provided `bltavares/home-consule` Docker image.

### Targets

| Platform         | Docker | Size   |
|------------------|--------|--------|
| armv7-musleabihf | Yes    | 4.69MB |
| arm-musleabi     | Yes    | 4.79MB |
| aarch64-musl     | Yes    | 4.69MB |
| x86_64-musl      | Yes    | 6.07MB |
| Windows          | No     | 5.81MB |
| Mac              | No     | N/A    |

### Docker

Example command for Docker

```shell
docker run -d \
  --restart=unless-stopped \
  -v ${PWD}:/app \
  -p 3000:3000 \
  -e CONSUL_HTTP_TOKEN \
  -e CONSUL_HTTP_ADDR \
  --name home \
  bltavares/home-consule
```

## Build

To build and publish multi-architecture docker images:

```shell
make all
make publish
make manifest
```
