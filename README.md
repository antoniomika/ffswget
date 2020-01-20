# ffswget

> A wget-able interface for Firefox Send

This project makes use of the great [ffsend-api](https://gitlab.com/timvisee/ffsend-api) by [Tim Visée](https://timvisee.com/)

## How to run this project

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

## How to use a running instance

1. Upload files

    - ```bash
      curl --progress-bar --upload-file ~/Downloads/ubuntu-19.04-live-server-amd64.iso http://<yourhost>:8000
      ```

2. Download files from a URL like <https://send.firefox.com/download/file_id/#file_key>

    - ```bash
      wget http://<yourhost>:8000/file_id/file_key -O file_name.iso
      ```

    - ```bash
      wget http://<yourhost>:8000/download?url=https%3A%2F%2Fsend.firefox.com%2Fdownload%2Ffile_id%2F%23file_key -O file_name.iso
      ```
