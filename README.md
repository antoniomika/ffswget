# ffswget

> A wget-able interface for Firefox Send

This project makes use of the great [ffsend-api](https://gitlab.com/timvisee/ffsend-api) by [Tim Visée](https://timvisee.com/)

## How to use this project

The command takes one argument, which is the base for the host of the URLs it returns.

### Docker

```bash
docker run -itd \
    -p 8000:8000 \
    --restart=always \
    antoniomika/ffswget:latest http://<yourhost>:8000
```

### Docker compose service example

```yaml
version: '3.7'

services:
  ffswget:
    image: antoniomika/ffswget:latest
    ports:
      - "8000:8000"
    command: http://<yourhost>:8000
    restart: always
```
